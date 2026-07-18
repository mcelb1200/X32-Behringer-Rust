use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "x32-cli",
    author,
    version,
    about = "Unified Behringer X32 CLI Control Tools"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Send custom OSC commands to the X32 console
    Command(x32_command::Args),
    /// Run x32_udp
    X32Udp(x32_udp::Args),
    /// Run xair_set_scene
    XairSetScene(xair_set_scene::Args),
    /// Run xair_get_scene
    XairGetScene(xair_get_scene::Args),
    /// Run x32_set_scene
    X32SetScene(x32_set_scene::Args),
    /// Run x32_get_scene
    X32GetScene(x32_get_scene::Args),
    /// Run x32_get_scene_name
    X32GetSceneName(x32_get_scene_name::Args),
    /// Run x32_cpxlivemarkers
    X32Cpxlivemarkers(x32_cpxlivemarkers::Cli),
    /// Run x32_tcp
    X32Tcp(x32_tcp::Args),
    /// Run x32_geq2_cpy
    X32Geq2Cpy(x32_geq2_cpy::Args),
    /// Run x32_jog4xlive
    X32Jog4xlive(x32_jog4xlive::Args),
    /// Run x32_dump
    X32Dump(x32_dump::Args),
    /// Run x32_xlive_wav
    X32XliveWav(x32_xlive_wav::Args),
    /// Run xair_command
    XairCommand(xair_command::Args),
    /// Run x32_copy_fx
    X32CopyFx(x32_copy_fx::Args),
    /// Run x32_get_lib
    X32GetLib(x32_get_lib::Args),
    /// Run x32_desk_restore
    X32DeskRestore(x32_desk_restore::Args),
    /// Run x32_desk_save
    X32DeskSave(x32_desk_save::Args),
    /// Run x32_ssavergw
    X32Ssavergw(x32_ssavergw::Args),
    /// Run x32_set_lib
    X32SetLib(x32_set_lib::Args),
    /// Run x32_punch_control
    X32PunchControl(x32_punch_control::Args),
    /// Run x32_set_preset
    X32SetPreset(x32_set_preset::Args),
    /// Run x32_replay
    X32Replay(x32_replay::Args),
    /// Run x32_fade
    X32Fade(x32_fade::Args),
    /// Run x32_custom_layer
    X32CustomLayer(x32_custom_layer::Cli),
    /// Run x32_wav_xlive
    X32WavXlive(x32_wav_xlive::Args),
    /// Run x32_commander
    X32Commander(x32_commander::Args),
    /// Run x32_loudness
    X32Loudness(x32_loudness::Cli),
    /// Run x32_automix
    X32Automix(x32_automix::Args),
    /// Run x32_sync
    X32Sync(x32_sync::Args),
    /// Run x32_tap
    X32Tap(x32_tap::Args),
    /// Run x32_tapw
    X32Tapw(x32_tapw::Args),
    /// Run x32_autobeat
    X32Autobeat(x32_autobeat::Cli),
    /// Run x32_dca_spill
    X32DcaSpill(x32_dca_spill::Args),
    /// Run x32_reaper
    X32Reaper(x32_reaper::Args),
    /// Run x32_vocal_ducking
    X32VocalDucking(x32_vocal_ducking::Cli),
    /// Run x32_usb
    X32Usb(x32_usb::Args),
    /// Run x32_midi2osc
    X32Midi2osc(x32_midi2osc::Args),
    /// Run x32_emulator
    X32Emulator(x32_emulator::Cli),
    /// Run x32_crossfade
    X32Crossfade(x32_crossfade::Args),
    /// Run x32_auto_gain
    X32AutoGain(x32_auto_gain::Args),
    /// Run x32_auto_ringout
    X32AutoRingout(x32_auto_ringout::Args),
    /// Run x32_feedback_detect
    X32FeedbackDetect(x32_feedback_detect::Args),
    /// Run x32_scene_checker
    X32SceneChecker(x32_scene_checker::Args),
    /// Run x32_system_tune
    X32SystemTune(x32_system_tune::Args),
    /// Run x32_volunteer
    X32Volunteer(x32_volunteer::Args),
    /// Run x32_safe_mute
    X32SafeMute(x32_safe_mute::Args),
    /// Run x32_speech_mode
    X32SpeechMode(x32_speech_mode::Args),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Command(args) => x32_command::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Udp(args) => x32_udp::run(args).await.map_err(anyhow::Error::msg),
        Commands::XairSetScene(args) => xair_set_scene::run(args).await.map_err(anyhow::Error::msg),
        Commands::XairGetScene(args) => xair_get_scene::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32SetScene(args) => x32_set_scene::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32GetScene(args) => x32_get_scene::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32GetSceneName(args) => x32_get_scene_name::run(args)
            .await
            .map_err(anyhow::Error::msg),
        Commands::X32Cpxlivemarkers(args) => {
            x32_cpxlivemarkers::run(args).map_err(anyhow::Error::msg)
        }
        Commands::X32Tcp(args) => x32_tcp::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Geq2Cpy(args) => x32_geq2_cpy::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Jog4xlive(args) => x32_jog4xlive::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Dump(args) => x32_dump::run(args).map_err(anyhow::Error::msg),
        Commands::X32XliveWav(args) => x32_xlive_wav::run(args).map_err(anyhow::Error::msg),
        Commands::XairCommand(args) => xair_command::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32CopyFx(args) => x32_copy_fx::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32GetLib(args) => x32_get_lib::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32DeskRestore(args) => x32_desk_restore::run(args)
            .await
            .map_err(anyhow::Error::msg),
        Commands::X32DeskSave(args) => x32_desk_save::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Ssavergw(args) => x32_ssavergw::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32SetLib(args) => x32_set_lib::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32PunchControl(args) => x32_punch_control::run(args)
            .await
            .map_err(anyhow::Error::msg),
        Commands::X32SetPreset(args) => x32_set_preset::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Replay(args) => x32_replay::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Fade(args) => x32_fade::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32CustomLayer(args) => x32_custom_layer::run(args)
            .await
            .map_err(anyhow::Error::msg),
        Commands::X32WavXlive(args) => x32_wav_xlive::run(args).map_err(anyhow::Error::msg),
        Commands::X32Commander(args) => x32_commander::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Loudness(args) => x32_loudness::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Automix(args) => x32_automix::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Sync(args) => x32_sync::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Tap(args) => x32_tap::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Tapw(args) => x32_tapw::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Autobeat(args) => x32_autobeat::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32DcaSpill(args) => x32_dca_spill::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Reaper(args) => x32_reaper::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32VocalDucking(args) => x32_vocal_ducking::run(args)
            .await
            .map_err(anyhow::Error::msg),
        Commands::X32Usb(args) => x32_usb::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Midi2osc(args) => x32_midi2osc::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32Emulator(args) => x32_emulator::run(args).map_err(anyhow::Error::msg),
        Commands::X32Crossfade(args) => x32_crossfade::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32AutoGain(args) => x32_auto_gain::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32AutoRingout(args) => x32_auto_ringout::run(args)
            .await
            .map_err(anyhow::Error::msg),
        Commands::X32FeedbackDetect(args) => x32_feedback_detect::run(args)
            .await
            .map_err(anyhow::Error::msg),
        Commands::X32SceneChecker(args) => x32_scene_checker::run(args)
            .await
            .map_err(anyhow::Error::msg),
        Commands::X32SystemTune(args) => {
            x32_system_tune::run(args).await.map_err(anyhow::Error::msg)
        }
        Commands::X32Volunteer(args) => x32_volunteer::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32SafeMute(args) => x32_safe_mute::run(args).await.map_err(anyhow::Error::msg),
        Commands::X32SpeechMode(args) => {
            x32_speech_mode::run(args).await.map_err(anyhow::Error::msg)
        }
    }
}
