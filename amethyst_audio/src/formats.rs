use amethyst_assets::*;

use serde::{Deserialize, Serialize};
use serde_dyn::{uuid};

use super::Source as Audio;

#[derive(Clone, Serialize, Deserialize)]
pub struct AudioData(pub Vec<u8>);
uuid!{
    AudioData => 152363896001301345872018713952728977845,
    WavFormat => 112532213756211251056778320838394330515,
    OggFormat => 112532283476994263609395403157070626195,
    FlacFormat => 112532310414569518459270184962013740435,
    Mp3Format => 112532334975299897881214838960638344595
}

/// Loads audio from wav files.
#[derive(Clone)]
pub struct WavFormat;

impl SimpleFormat<Audio> for WavFormat {
    fn name() -> &'static str { "WAV"}

    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<AudioData> {
        Ok(AudioData(bytes))
    }
}

/// Loads audio from Ogg Vorbis files
#[derive(Clone)]
pub struct OggFormat;

impl SimpleFormat<Audio> for OggFormat {
    fn name() -> &'static str {"OGG"}

    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<AudioData> {
        Ok(AudioData(bytes))
    }
}

/// Loads audio from Flac files.
#[derive(Clone)]
pub struct FlacFormat;

impl SimpleFormat<Audio> for FlacFormat {
    fn name() -> &'static str {"FLAC"}

    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<AudioData> {
        Ok(AudioData(bytes))
    }
}

/// Loads audio from MP3 files.
#[derive(Clone)]
pub struct Mp3Format;

impl SimpleFormat<Audio> for Mp3Format {
    fn name() -> &'static str { "MP3"}

    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<AudioData> {
        Ok(AudioData(bytes))
    }
}
/// Aggregate sound format
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AudioFormat {
    /// Ogg
    Ogg,
    /// Wav
    Wav,
    /// Flac
    Flac,
    /// Mp3
    Mp3,
}

impl SimpleFormat<Audio> for AudioFormat {
    fn name() -> &'static str { "AudioFormat"}

    type Options = ();

    fn import(&self, bytes: Vec<u8>, options: ()) -> Result<AudioData> {
        match *self {
            AudioFormat::Ogg => SimpleFormat::import(&OggFormat, bytes, options),
            AudioFormat::Wav => SimpleFormat::import(&WavFormat, bytes, options),
            AudioFormat::Flac => SimpleFormat::import(&FlacFormat, bytes, options),
            AudioFormat::Mp3 => SimpleFormat::import(&Mp3Format, bytes, options),
        }
    }
}
