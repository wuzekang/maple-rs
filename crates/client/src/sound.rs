use crate::{wz::WzSplitReaderExt, WzSplitReaderContext};
use sdl3_sys::everything::*;
use std::ffi::c_void;
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::CODEC_TYPE_NULL;
use symphonia::core::errors::Error;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use ::ui::reactive::{on_cleanup, use_context};
use ::ui::resource::use_resource;
use wz_parser::WzNodeCast;

struct AudioStream(*mut SDL_AudioStream);

impl AudioStream {
    pub(crate) fn pause(&self) {
        unsafe {
            SDL_PauseAudioStreamDevice(self.0);
        }
    }
}

unsafe impl Send for AudioStream {}

pub fn play_sound(path: &str) {
    let WzSplitReaderContext { reader } = use_context().unwrap();
    let path = path.to_string();

    let cancelled = Arc::new(AtomicBool::new(false));
    let (tx, rx) = std::sync::mpsc::channel::<AudioStream>();

    on_cleanup({
        let cancelled = cancelled.clone();
        move || {
            cancelled.store(true, Ordering::Relaxed);
            if let Ok(stream) = rx.try_recv() {
                stream.pause()
            }
        }
    });

    use_resource(
        move || {
            let reader = reader.clone();
            let path = path.clone();
            let cancelled = cancelled.clone();
            let tx = tx.clone();
            async move {
                let sound_node = match reader.get_node(&path).await {
                    Ok(node) => node,
                    Err(_) => return,
                };
                let node = sound_node.wz_node.read().unwrap();
                let sound = node.try_as_sound().unwrap();
                let buffer = sound.get_buffer();
                // Probe the media source.
                let probed = symphonia::default::get_probe()
                    .format(
                        Hint::new().with_extension("mp3"),
                        // Create the media source stream.
                        MediaSourceStream::new(Box::new(Cursor::new(buffer)), Default::default()),
                        // Use the default options for metadata and format readers.
                        &Default::default(),
                        &Default::default(),
                    )
                    .expect("unsupported format");

                // Get the instantiated format reader.
                let mut format = probed.format;

                // Find the first audio track with a known (decodeable) codec.
                let track = format
                    .tracks()
                    .iter()
                    .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
                    .expect("no supported audio tracks");

                // Create a decoder for the track.
                let mut decoder = symphonia::default::get_codecs()
                    // Use the default options for the decoder.
                    .make(&track.codec_params, &Default::default())
                    .expect("unsupported codec");

                // Store the track identifier, it will be used to filter packets.
                let track_id = track.id;

                let mut sample_buf = None;
                let mut sdl_stream = None;

                // The decode loop.
                loop {
                    if cancelled.load(Ordering::Relaxed) {
                        return;
                    }
                    // Get the next packet from the media format.
                    let packet = match format.next_packet() {
                        Ok(packet) => packet,
                        Err(Error::ResetRequired) => {
                            // The track list has been changed. Re-examine it and create a new set of decoders,
                            // then restart the decode loop. This is an advanced feature and it is not
                            // unreasonable to consider this "the end." As of v0.5.0, the only usage of this is
                            // for chained OGG physical streams.
                            unimplemented!();
                        }
                        Err(Error::IoError(_err)) => {
                            // A unrecoverable error occurred, halt decoding.
                            // format!("{}", err);
                            break;
                        }
                        Err(err) => {
                            // A unrecoverable error occurred, halt decoding.
                            panic!("{}", err);
                        }
                    };

                    // Consume any new metadata that has been read since the last packet.
                    while !format.metadata().is_latest() {
                        // Pop the old head of the metadata queue.
                        format.metadata().pop();
                        // Consume the new metadata at the head of the metadata queue.
                    }

                    // If the packet does not belong to the selected track, skip over it.
                    if packet.track_id() != track_id {
                        continue;
                    }

                    // Decode the packet into audio samples.
                    match decoder.decode(&packet) {
                        Ok(audio_buf) => {
                            // The decoded audio samples may now be accessed via the audio buffer if per-channel
                            // slices of samples in their native decoded format is desired. Use-cases where
                            // the samples need to be accessed in an interleaved order or converted into
                            // another sample format, or a byte buffer is required, are covered by copying the
                            // audio buffer into a sample buffer or raw sample buffer, respectively. In the
                            // example below, we will copy the audio buffer into a sample buffer in an
                            // interleaved order while also converting to a f32 sample format.

                            // If this is the *first* decoded packet, create a sample buffer matching the
                            // decoded audio buffer format.
                            if sample_buf.is_none() {
                                // Get the audio buffer specification.
                                let spec = *audio_buf.spec();

                                // Get the capacity of the decoded buffer. Note: This is capacity, not length!
                                let duration = audio_buf.capacity() as u64;

                                // Create the f32 sample buffer.
                                sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                            }

                            if sdl_stream.is_none() {
                                let spec = *audio_buf.spec();
                                let stream = unsafe {
                                    SDL_OpenAudioDeviceStream(
                                        SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK,
                                        &SDL_AudioSpec {
                                            format: SDL_AUDIO_F32,
                                            channels: spec.channels.count() as i32,
                                            freq: spec.rate as i32,
                                        },
                                        None,
                                        std::ptr::null_mut(),
                                    )
                                };

                                unsafe {
                                    SDL_ResumeAudioStreamDevice(stream);
                                }

                                tx.send(AudioStream(stream)).unwrap();

                                sdl_stream = Some(stream);
                            }

                            // Copy the decoded audio buffer into the sample buffer in an interleaved format.
                            if let Some(buf) = &mut sample_buf {
                                buf.copy_interleaved_ref(audio_buf);

                                if let Some(stream) = sdl_stream {
                                    unsafe {
                                        SDL_PutAudioStreamData(
                                            stream,
                                            buf.samples().as_ptr() as *const c_void,
                                            (buf.samples().len() * 4) as i32,
                                        );
                                    };
                                }
                            }
                        }
                        Err(Error::IoError(_)) => {
                            // The packet failed to decode due to an IO error, skip the packet.
                            continue;
                        }
                        Err(Error::DecodeError(_)) => {
                            // The packet failed to decode due to invalid data, skip the packet.
                            continue;
                        }
                        Err(err) => {
                            // An unrecoverable error occurred, halt decoding.
                            panic!("{}", err);
                        }
                    }
                }
            }
        },
        move |_| {},
    );
}
