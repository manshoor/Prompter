use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct AudioData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl AudioData {
    /// Convert to mono 16kHz WAV bytes (required by Whisper)
    pub fn to_wav_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Convert to mono if multi-channel
        let mono_samples: Vec<f32> = if self.channels > 1 {
            self.samples
                .chunks(self.channels as usize)
                .map(|chunk| chunk.iter().sum::<f32>() / chunk.len() as f32)
                .collect()
        } else {
            self.samples.clone()
        };

        // Resample to 16kHz if needed (linear interpolation)
        let target_rate = 16000u32;
        let resampled = if self.sample_rate != target_rate {
            let ratio = self.sample_rate as f64 / target_rate as f64;
            let new_len = (mono_samples.len() as f64 / ratio) as usize;
            (0..new_len)
                .map(|i| {
                    let src_idx = i as f64 * ratio;
                    let idx = src_idx as usize;
                    let frac = src_idx - idx as f64;
                    let s1 = mono_samples.get(idx).copied().unwrap_or(0.0);
                    let s2 = mono_samples.get(idx + 1).copied().unwrap_or(s1);
                    s1 + (s2 - s1) * frac as f32
                })
                .collect()
        } else {
            mono_samples
        };

        let mut cursor = std::io::Cursor::new(Vec::new());
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: target_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::new(&mut cursor, spec)?;
        for sample in &resampled {
            let s = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(s)?;
        }
        writer.finalize()?;
        Ok(cursor.into_inner())
    }
}

pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    stream: Mutex<Option<cpal::Stream>>,
    sample_rate: Mutex<u32>,
    channels: Mutex<u16>,
    /// Timestamp (in samples) of the last chunk with audio above silence threshold
    last_voice_sample: Arc<Mutex<u64>>,
    /// Total samples received
    total_samples: Arc<Mutex<u64>>,
    /// Current RMS audio level (updated each audio callback)
    current_rms: Arc<Mutex<f32>>,
}

// SAFETY: AudioRecorder fields are all protected by Mutex. The cpal::Stream is not Send/Sync
// due to platform-specific marker types, but we only access it through the Mutex, ensuring
// exclusive access. The stream is created and dropped while holding the lock.
unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

impl AudioRecorder {
    /// RMS threshold below which audio is considered silence
    const SILENCE_RMS_THRESHOLD: f32 = 0.01;

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: Mutex::new(None),
            sample_rate: Mutex::new(0),
            channels: Mutex::new(0),
            last_voice_sample: Arc::new(Mutex::new(0)),
            total_samples: Arc::new(Mutex::new(0)),
            current_rms: Arc::new(Mutex::new(0.0)),
        })
    }

    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;
        let config = device.default_input_config()?;

        *self.sample_rate.lock().unwrap() = config.sample_rate().0;
        *self.channels.lock().unwrap() = config.channels();

        let samples = Arc::clone(&self.samples);
        samples.lock().unwrap().clear();

        let last_voice = Arc::clone(&self.last_voice_sample);
        let total = Arc::clone(&self.total_samples);
        let rms_out = Arc::clone(&self.current_rms);
        *last_voice.lock().unwrap() = 0;
        *total.lock().unwrap() = 0;
        *rms_out.lock().unwrap() = 0.0;

        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                samples.lock().unwrap().extend_from_slice(data);

                // Check if this chunk has voice (above silence threshold)
                let rms = (data.iter().map(|s| s * s).sum::<f32>() / data.len() as f32).sqrt();
                *rms_out.lock().unwrap() = rms;
                let mut total_count = total.lock().unwrap();
                *total_count += data.len() as u64;
                if rms > Self::SILENCE_RMS_THRESHOLD {
                    *last_voice.lock().unwrap() = *total_count;
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        stream.play()?;
        *self.stream.lock().unwrap() = Some(stream);
        Ok(())
    }

    /// Returns how many seconds of silence have elapsed since the last detected voice.
    pub fn silence_duration_secs(&self) -> f32 {
        let sample_rate = *self.sample_rate.lock().unwrap();
        if sample_rate == 0 {
            return 0.0;
        }
        let total = *self.total_samples.lock().unwrap();
        let last_voice = *self.last_voice_sample.lock().unwrap();
        let silent_samples = total.saturating_sub(last_voice);
        silent_samples as f32 / sample_rate as f32
    }

    /// Returns the current RMS audio level (0.0 = silence, 1.0 = max).
    pub fn current_rms(&self) -> f32 {
        *self.current_rms.lock().unwrap()
    }

    pub fn stop(&self) -> Result<AudioData, Box<dyn std::error::Error>> {
        *self.stream.lock().unwrap() = None;

        let samples = std::mem::take(&mut *self.samples.lock().unwrap());
        let sample_rate = *self.sample_rate.lock().unwrap();
        let channels = *self.channels.lock().unwrap();

        Ok(AudioData {
            samples,
            sample_rate,
            channels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    #[ignore] // Requires microphone access; run manually with --ignored
    fn test_recorder_starts_and_stops() {
        let recorder = AudioRecorder::new().unwrap();
        recorder.start().unwrap();
        thread::sleep(Duration::from_millis(500));
        let audio_data = recorder.stop().unwrap();
        assert!(!audio_data.samples.is_empty());
        assert!(audio_data.sample_rate > 0);
    }

    #[test]
    #[ignore] // Requires microphone access; run manually with --ignored
    fn test_recorder_returns_valid_sample_rate() {
        let recorder = AudioRecorder::new().unwrap();
        recorder.start().unwrap();
        thread::sleep(Duration::from_millis(200));
        let audio_data = recorder.stop().unwrap();
        assert!(audio_data.sample_rate >= 8000);
    }

    #[test]
    fn test_audio_data_to_wav_bytes_mono() {
        // Synthesize a short sine wave at 16kHz mono
        let sample_rate = 16000u32;
        let duration_secs = 0.1;
        let num_samples = (sample_rate as f64 * duration_secs) as usize;
        let samples: Vec<f32> = (0..num_samples)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sample_rate as f32).sin())
            .collect();

        let audio_data = AudioData {
            samples,
            sample_rate,
            channels: 1,
        };

        let wav_bytes = audio_data.to_wav_bytes().unwrap();
        // WAV header starts with "RIFF"
        assert_eq!(&wav_bytes[0..4], b"RIFF");
        // Format chunk contains "WAVE"
        assert_eq!(&wav_bytes[8..12], b"WAVE");
        assert!(wav_bytes.len() > 44); // header + some data
    }

    #[test]
    fn test_audio_data_to_wav_bytes_stereo_downmix() {
        // Stereo signal: left=1.0, right=-1.0 => mono average = 0.0
        let sample_rate = 16000u32;
        let num_frames = 1600; // 0.1s at 16kHz
        let mut samples = Vec::with_capacity(num_frames * 2);
        for _ in 0..num_frames {
            samples.push(1.0f32); // left
            samples.push(-1.0f32); // right
        }

        let audio_data = AudioData {
            samples,
            sample_rate,
            channels: 2,
        };

        let wav_bytes = audio_data.to_wav_bytes().unwrap();
        assert_eq!(&wav_bytes[0..4], b"RIFF");

        // Read back and verify samples are ~0
        let cursor = std::io::Cursor::new(wav_bytes);
        let mut reader = hound::WavReader::new(cursor).unwrap();
        let read_samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(read_samples.len(), num_frames);
        for s in &read_samples {
            assert_eq!(*s, 0);
        }
    }

    #[test]
    fn test_audio_data_to_wav_bytes_resamples() {
        // 48kHz mono signal resampled to 16kHz
        let sample_rate = 48000u32;
        let duration_secs = 0.1;
        let num_samples = (sample_rate as f64 * duration_secs) as usize; // 4800
        let samples: Vec<f32> = (0..num_samples).map(|_| 0.5).collect();

        let audio_data = AudioData {
            samples,
            sample_rate,
            channels: 1,
        };

        let wav_bytes = audio_data.to_wav_bytes().unwrap();
        let cursor = std::io::Cursor::new(wav_bytes);
        let reader = hound::WavReader::new(cursor).unwrap();
        let spec = reader.spec();
        assert_eq!(spec.sample_rate, 16000);
        assert_eq!(spec.channels, 1);
        // Expected ~1600 samples (4800 / 3)
        let read_samples: Vec<i16> = reader.into_samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(read_samples.len(), 1600);
    }
}
