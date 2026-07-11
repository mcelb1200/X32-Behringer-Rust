# TODO — Feature Roadmap

> **Audience note:** These features are designed to be operated by novice and volunteer
> sound operators (e.g., church volunteers, school AV club members, community event helpers)
> who may have little or no formal audio training. Every feature should default to *safe*
> behaviour and clearly explain what it is doing and why.

---

## 1. Automatic Feedback Detection and Management

### Why this matters (plain language)
Feedback is the painful squealing or ringing sound that happens when a microphone picks up
its own amplified signal from a speaker. It's the #1 fear of volunteer sound operators and
the #1 cause of audience discomfort. This feature acts like an always-on safety net that
catches feedback before it becomes audible.

### How it works — technical approach
| Stage | Detail |
|---|---|
| **Detection** | Continuously analyse the audio spectrum using FFT (Fast Fourier Transform). Feedback appears as an abnormally narrow, sustained peak that rises faster than musical content. Use a **peak-to-average ratio** and **temporal persistence** test: a genuine feedback tone will persist for >150 ms at a Q > 30, whereas musical notes are broader and transient. |
| **Classification** | Distinguish between *incipient* feedback (a ringing resonance that hasn't yet become self-sustaining) and *full* feedback (a runaway howl). Incipient feedback can be tamed with a shallow notch (−3 to −6 dB); full feedback requires a deeper cut (−9 to −12 dB) plus an immediate level reduction. |
| **Intervention** | Insert a surgical **parametric EQ (PEQ) notch filter** on the affected channel or bus. The X32 provides 6 bands of fully parametric EQ per channel (`/ch/XX/eq/1–6`) and 6 per bus (`/bus/XX/eq/1–6`). Target the narrowest Q possible (typically 10–40) to remove the feedback frequency while preserving tonal quality. |
| **Data source** | Primary: USB audio return stream (the X32 can send all 32 channels + aux over USB as a 32×32 audio interface — use this for high-resolution FFT when available). Fallback: meter data from `/meters` OSC subscriptions (lower resolution, but works over Ethernet with no USB connection). |

### X32 OSC paths involved
- `/ch/XX/eq/N/type` — set to PEQ (parametric) = `3`
- `/ch/XX/eq/N/freq` — centre frequency (scaled 20 Hz–20 kHz, float 0.0–1.0)
- `/ch/XX/eq/N/gain` — notch depth in dB (float, −15.0 to +15.0)
- `/ch/XX/eq/N/q` — bandwidth / Q factor (float 0.3–10.0 mapped)
- `/meters` — subscribe to metering blocks for RMS/peak analysis

### Safety guardrails
- **Max notch depth cap**: Never cut more than −12 dB per band automatically. If feedback persists after maximum intervention, reduce the channel fader by −6 dB and alert the operator rather than stacking more notches.
- **Notch limit**: Use at most 3 of the 6 available EQ bands for automatic notches, reserving the remaining bands for the operator's manual EQ shaping.
- **Undo / reset**: Always store the original EQ state before modifying. Provide a one-tap "undo all feedback notches" action.
- **Visual feedback**: Clearly display which frequencies were notched on the TUI with an indicator like `🔇 Notch @ 2.4 kHz (−6 dB)` so operators can learn what's happening.

### Real-world use case
> A church volunteer is running sound for a Sunday service. The pastor walks in front of
> a floor monitor and a 3.1 kHz ring starts building. The system detects the rising peak
> within 100 ms, inserts a −6 dB notch at 3.1 kHz on the pastor's channel EQ, and the
> ring disappears before the congregation notices. The TUI briefly flashes an amber
> notification: *"Feedback caught @ 3.1 kHz on Ch 05 (Pastor)"*.

---

## 2. Auto-Gain / Smart Gain Staging

### Why this matters (plain language)
"Gain staging" is the process of setting how much the mixer amplifies a microphone's raw
signal before any other processing. Set it too low and the signal is buried in noise. Set
it too high and it clips (distorts). Getting this right is the foundation of good sound,
but it requires experience that volunteers typically don't have. This feature does it for
them automatically.

### How it works — technical approach

#### Phase 1: Measurement
1. Subscribe to `/meters` to read the pre-fader channel meter levels (meter block index 0 — input/preamp levels).
2. During a configurable measurement window (default: 5 seconds of active signal), collect peak and RMS levels.
3. Calculate the **crest factor** (peak-to-RMS ratio) to understand the signal's dynamic character.

#### Phase 2: Target selection — instrument-aware optimal levels
The system infers the instrument/source type from the X32's **scribble strip** data:
- `/ch/XX/config/name` — user-assigned channel name (e.g., `"Kick"`, `"Pastor"`, `"Acoustic Gtr"`)
- `/ch/XX/config/icon` — preset icon ID (the X32 has ~74 preset icons: drum icons, mic icons, guitar icons, etc.)
- `/ch/XX/config/color` — colour coding (can correlate with common conventions like warm colours for drums)

| Source type (matched by name/icon) | Target RMS | Target peak | Crest factor | Notes |
|---|---|---|---|---|
| Kick drum / Bass drum | −12 dBFS | −6 dBFS | ~6 dB | High transient energy, needs headroom |
| Snare drum | −14 dBFS | −6 dBFS | ~8 dB | Very high peaks from rimshots |
| Toms | −14 dBFS | −6 dBFS | ~8 dB | Similar to snare profile |
| Overhead / Cymbal | −18 dBFS | −10 dBFS | ~8 dB | Wide dynamic range |
| Bass guitar (DI) | −14 dBFS | −8 dBFS | ~6 dB | Consistent, moderate dynamics |
| Electric guitar (amp mic) | −16 dBFS | −8 dBFS | ~8 dB | Varies with distortion level |
| Acoustic guitar (mic) | −18 dBFS | −10 dBFS | ~8 dB | Gentle dynamics, fingerpicking |
| Piano / Keys (DI/stereo) | −18 dBFS | −8 dBFS | ~10 dB | Very wide dynamic range |
| Lead vocal | −18 dBFS | −8 dBFS | ~10 dB | Highly dynamic, must avoid clipping during belting |
| Speech / Lectern | −20 dBFS | −10 dBFS | ~10 dB | Conservative — unexpected shouts, mic handling noise |
| Choir / Ensemble (stereo) | −20 dBFS | −10 dBFS | ~10 dB | Large dynamic swings between pp and ff |
| Wireless lavalier | −22 dBFS | −12 dBFS | ~10 dB | Prone to handling noise and clothing rustle |
| DJ / Playback (line level) | −14 dBFS | −6 dBFS | ~8 dB | Pre-compressed, predictable levels |
| **Best-guess (unknown)** | **−18 dBFS** | **−10 dBFS** | **~8 dB** | **Safe default — leaves generous headroom** |

#### Phase 3: Gain adjustment
- Read current HA (head amplifier) gain: `/ch/XX/preamp/trim` (float 0.0–1.0, mapping to −12 dB to +60 dB on mic inputs, −12 dB to +12 dB on line inputs)
- Calculate required offset = target level − measured level
- Apply the new gain value. **Never jump more than 6 dB per step** — ramp in 3 dB increments with 200 ms delays to avoid audible jumps.
- For **stereo-linked channels** (`/ch/XX/preamp/link` = 1 or stereo bus pairs), apply identical gain to both L and R channels to preserve stereo imaging.

### X32 OSC paths involved
- `/ch/XX/preamp/trim` — head amplifier gain
- `/ch/XX/preamp/link` — stereo link state
- `/ch/XX/config/name` — scribble strip name
- `/ch/XX/config/icon` — scribble strip icon ID
- `/ch/XX/meter` or `/meters` block — level metering

### Safety guardrails
- **Never set gain while signal is absent** — require at least 2 seconds of signal above −60 dBFS before making adjustments.
- **Clip protection**: If any peak exceeds −3 dBFS during adjustment, immediately reduce gain by 6 dB and restart measurement.
- **Phantom power awareness**: Log a warning if phantom power (`/ch/XX/preamp/+48V`) state changes during gain adjustment, as this can cause transients.
- **User confirmation**: In cautious mode, display the proposed gain change and wait for operator approval before applying.

### Real-world use case
> A school AV club student is setting up for a talent show. They've plugged in 16 microphones
> but have no idea what gain levels to use. They run Auto-Gain, which reads the scribble strips
> ("Vocal 1", "Guitar Amp", "Kick", etc.), asks each performer to play/speak for 5 seconds, and
> automatically dials in the preamp gain. The student sees a progress display:
> `Ch 01 [Kick] ████████░░ −12 dBFS ✓`

---

## 3. Auto-Ringout

### Why this matters (plain language)
"Ringing out" a monitor (foldback wedge or in-ear mix) is the process of finding and removing
the frequencies that are most likely to cause feedback in that particular monitor. It's a
skilled, time-consuming task that professional sound engineers do before every show. This
feature automates that entire process so a volunteer can safely turn up monitors without fear.

### How it works — technical approach

#### Pre-requisites
- The operator selects which bus(es) to ring out (e.g., Bus 01 = "Monitor L", Bus 05 = "Drum fill").
- The system needs an open microphone on stage pointed at (or near) the monitor being tested.
- All other monitors should be muted or significantly lowered during ringout.

#### Ringout sequence
1. **Arm** the selected bus for ringout. The system stores the current bus EQ state and aux send levels as a restore point.
2. **Raise the bus level gradually** (in 1 dB increments, 500 ms apart) from the current level toward the target operating level. The system uses the **Auto-Gain** function to determine the target aux send level based on instrument type (from scribble strip data).
3. **Listen for feedback onset** using the **Automatic Feedback Detection** function. When a resonant peak is detected:
   - Log the frequency, amplitude, and Q.
   - Insert a notch filter on the **bus EQ** (not the channel EQ — ringout notches belong on the monitor bus so they don't affect the front-of-house mix).
   - Continue raising the level.
4. **Repeat** until the bus reaches the target operating level or until the maximum number of notch filters (typically 5–6 per bus) is exhausted.
5. **Dynamic re-evaluation**: When new channels are added to the bus (e.g., a new musician walks on stage), or when send levels are increased, the system re-arms and performs an incremental ringout — testing only the new/changed contributions without re-doing the entire process.

#### Dugan automixer integration
For buses carrying speech microphones, the system configures the **Dugan automixer** to share gain across active microphones. This reduces the cumulative open-mic gain and provides inherent feedback margin:
- The X32's Dugan automixer is available on Insert FX slots.
- Auto-Ringout sets the automixer's channel weights based on the instrument type and expected signal levels.

### X32 OSC paths involved
- `/bus/XX/eq/N/*` — bus parametric EQ for notch insertion
- `/bus/XX/mix/fader` — bus master fader level
- `/ch/XX/mix/NN/level` — per-channel send level to bus NN
- `/ch/XX/mix/NN/on` — per-channel send on/off to bus NN
- `/bus/XX/insert/*` — insert effect routing (for Dugan automixer)
- `/fx/N/*` — FX slot parameters (Dugan config)

### TUI display during ringout
```
╔══════════════════════════════════════════════════╗
║  AUTO-RINGOUT                                    ║
║──────────────────────────────────────────────────║
║  Bus 01 [Monitor L]   ████████░░  ACTIVE         ║
║    Notch 1: 800 Hz   (−6 dB, Q=20)              ║
║    Notch 2: 2.4 kHz  (−9 dB, Q=15)              ║
║    Notch 3: — waiting —                          ║
║    Headroom remaining: +4 dB                     ║
║                                                  ║
║  Bus 02 [Monitor R]   ░░░░░░░░░░  ARMED          ║
║  Bus 05 [Drum fill]   ░░░░░░░░░░  DISARMED       ║
║──────────────────────────────────────────────────║
║  [A]rm/disarm  [P]ause  [R]eset  [Q]uit         ║
╚══════════════════════════════════════════════════╝
```

### Safety guardrails
- **Volume limit**: Never raise a bus fader above −6 dB during automated ringout (configurable). The gap between ringout level and performance level provides a safety margin called "gain before feedback."
- **Timeout**: If feedback is detected but cannot be resolved after 3 notch attempts at the same frequency range, pause the ringout and alert the operator — there may be a physical problem (mic placement, damaged speaker, etc.).
- **Preserve FOH**: Notches are applied only to bus EQ, never to channel EQ, so the front-of-house mix is not degraded.
- **Graceful abort**: Pressing the panic button (see §5) during ringout immediately restores all EQ and level changes.

### Real-world use case
> A community theatre volunteer is setting up 4 floor monitors for a musical. They select
> all 4 monitor buses, press "Auto-Ringout," and walk away. Five minutes later, each monitor
> has 3–5 surgical notches and the system reports *"All buses ringed out. Average gain before
> feedback: +6 dB above target level."* During the show, when a new actor's mic is turned up
> into Monitor 2, the system detects the change and performs a quick incremental ringout
> between scenes.

---

## 4. One-Touch "Speech Mode" Macro

### Why this matters (plain language)
The most common task for volunteer sound operators is mixing spoken word — sermons, lectures,
presentations, school announcements. This macro turns the X32 into an optimised speech
mixing system with a single button press. No knob-twiddling, no menu-diving. It applies
decades of broadcast audio engineering best practices automatically.

### What it does — detailed breakdown

| Processing block | Setting | Why |
|---|---|---|
| **High-pass filter** | 80 Hz, 18 dB/oct slope | Removes low-frequency rumble from HVAC, foot stomps, podium bumps, and handling noise. 80 Hz is safe for all human voices (fundamental of the deepest bass voice is ~85 Hz). |
| **Low-pass filter** | 12 kHz, 12 dB/oct slope | Gently rolls off sibilance and high-frequency noise above the useful speech range. |
| **Presence boost** | +3 dB shelf at 3–5 kHz | Adds intelligibility and "cut through" for speech in reverberant rooms (churches, gyms). This is the critical speech intelligibility range defined by the **Speech Transmission Index (STI)**. |
| **Low-mid scoop** | −2 dB at 250–400 Hz, Q=1.5 | Reduces "boxiness" and proximity effect common with close-mic'd speech, especially lavalier and headset microphones. |
| **Compressor** | Ratio 3:1, threshold −20 dBFS, attack 10 ms, release 100 ms, knee soft | Gently levels out the difference between a whisper and a shout. Prevents the PA from blasting when a speaker gets excited, and lifts quiet moments so they're still audible. |
| **Gate/Expander** | Threshold −50 dBFS, range −20 dB, attack 0.5 ms, release 200 ms | Reduces ambient noise pickup when the speaker pauses. Gentle enough to not cut off quiet speech. |
| **Dugan automixer** | Enabled across all speech channels | Automatically shares gain between multiple open microphones. When one person talks, their mic gets full gain; others are attenuated. This is the industry standard for panel discussions, Q&A sessions, and multi-pastor services. |
| **Auto-Gain** | Target: −20 dBFS RMS (speech profile) | Sets preamp gain to the conservative speech target from the instrument table (§2). |
| **Auto-Ringout** | Armed on all monitor buses receiving speech channels | Provides ongoing feedback protection with automatic arming/disarming per bus. |

### X32 OSC paths involved
- `/ch/XX/eq/1/type` = `5` (high-pass), `/ch/XX/eq/1/freq` = 80 Hz
- `/ch/XX/eq/6/type` = `6` (low-pass), `/ch/XX/eq/6/freq` = 12 kHz
- `/ch/XX/eq/3/type` = `3` (PEQ), `/ch/XX/eq/3/freq` = 3.5 kHz, `/ch/XX/eq/3/gain` = +3.0
- `/ch/XX/eq/2/type` = `3` (PEQ), `/ch/XX/eq/2/freq` = 300 Hz, `/ch/XX/eq/2/gain` = −2.0
- `/ch/XX/dyn/on` = 1, `/ch/XX/dyn/mode` = `COMP`, `/ch/XX/dyn/ratio` = 3.0
- `/ch/XX/dyn/thr` = −20.0, `/ch/XX/dyn/attack` = 10, `/ch/XX/dyn/release` = 100
- `/ch/XX/gate/on` = 1, `/ch/XX/gate/thr` = −50.0
- `/ch/XX/preamp/trim` — via Auto-Gain
- `/bus/XX/eq/*` — via Auto-Ringout

### Safety guardrails
- **Non-destructive**: Stores the complete channel state before applying. One-tap "restore" undoes everything.
- **Channel selection**: Only applies to channels explicitly selected by the operator. Never touches instrument channels.
- **Idempotent**: Pressing Speech Mode again disengages it (toggles off), restoring original state.

### Real-world use case
> A church volunteer arrives 30 minutes before the service. They power on the X32, select the
> 4 speech microphone channels (pastor, worship leader, lectern, roaming), and press
> **"Speech Mode"**. The system applies EQ, compression, gating, enables Dugan automixing,
> runs auto-gain during soundcheck prayer, and arms auto-ringout. The volunteer's only
> remaining task is adjusting the main fader to taste. Total setup time: 2 minutes.

---

## 5. "Safe Mute" / Panic Button

### Why this matters (plain language)
Things go wrong during live events — a microphone falls, someone trips over a cable, a phone
rings through the PA, or a child grabs a mic and screams into it. The panic button is a
*single key* that instantly silences the entire system safely. The key word is **safely** —
simply yanking the main fader to zero can cause a loud "pop" or "thump" through the speakers
that can damage equipment and startle the audience.

### How it works — technical approach

#### Fade-out design
A proper emergency mute uses a **rapid exponential fade** rather than an instantaneous mute:
1. Ramp all selected outputs from current level to −∞ over **250 ms** using an exponential curve (−20 dB in the first 50 ms, then the remaining tail). This is fast enough to stop any emergency within a quarter-second, but gradual enough to avoid the transient "pop" caused by a hard mute.
2. After the fade completes, engage the hardware mute on all outputs (`/main/st/mix/on` = 0, `/bus/XX/mix/on` = 0, `/mtx/XX/mix/on` = 0) as a hard safety backstop.

#### Scope options
| Mode | What gets muted | Use case |
|---|---|---|
| **All outputs** | Main L/R, all buses, all matrices, all aux outputs | Full panic — nuclear option |
| **Selected DCA groups** | Only channels assigned to specified DCA groups | Mute just "band" or just "speech" while keeping the other running |
| **Monitors only** | All bus outputs (not main L/R) | Feedback emergency on stage, audience unaffected |
| **Main only** | Main L/R and matrices | Stage monitoring continues, audience gets silence |

#### Recovery
- Press the panic button again (or a separate "restore" key) to **fade back in** over 1 second to the pre-panic levels.
- Display a confirmation prompt: *"Restore all outputs? [Y/N]"* to prevent accidental re-engagement.

### X32 OSC paths involved
- `/main/st/mix/fader` — main stereo fader
- `/main/st/mix/on` — main stereo mute
- `/bus/XX/mix/fader` — bus faders (01–16)
- `/bus/XX/mix/on` — bus mutes
- `/mtx/XX/mix/fader` — matrix faders (01–06)
- `/mtx/XX/mix/on` — matrix mutes
- `/dca/N/fader` — DCA group faders (1–8)
- `/dca/N/on` — DCA group mutes

### Safety guardrails
- **Dedicated keybinding**: Map to an unmistakable key (e.g., `Escape`, or a physical USB MIDI button) — not something that can be accidentally triggered.
- **State snapshot**: Before executing, save the complete fader/mute state so recovery is exact, not a guess.
- **Fade-out, never hard mute first**: Always fade first, then mute. This protects speakers (especially tweeters and compression drivers) from DC transients.
- **Works offline**: Panic should function even if the X32 network connection is degraded — queue the mute commands and retry.
- **Logging**: Log every panic event with timestamp for post-event review.

### Real-world use case
> During a school assembly, a wireless microphone's battery dies and produces a loud burst of
> static through the PA. A student volunteer hits `Esc` on the laptop running the TUI. Within
> 250 ms, all outputs fade to silence. The audience hears a brief fading crackle instead of a
> sustained blast. The student changes the battery, presses `Esc` again, confirms "Restore?",
> and the system fades back in smoothly.

---

## 6. Intelligent Scene Pre-flight Checker

### Why this matters (plain language)
The X32's "scene" system lets you save and recall complete mixer configurations — like
snapshots. But loading a scene can be dangerous: it might change which outputs are active,
swap routing so microphones go to the wrong speakers, or dramatically change gain/EQ settings
causing sudden feedback or silence. A volunteer who loads the wrong scene (or the right scene
at the wrong time) can kill the PA mid-service. This tool is a safety inspection that
runs *before* loading, telling you exactly what will change.

### How it works — technical approach

#### Diff engine
1. **Capture current state**: Read all critical parameters from the live mixer via OSC.
2. **Parse incoming scene**: Read the `.scn` file (X32 scene format — essentially a list of OSC parameter assignments).
3. **Compare**: Generate a structured diff of every parameter that would change.
4. **Classify risk**: Assign a risk level to each change based on its potential impact.

#### Risk classification matrix

| Change type | Risk level | Rationale |
|---|---|---|
| Routing changes (`/ch/XX/config/source`, `/routing/*`) | 🔴 **CRITICAL** | Can completely silence or mis-route the PA. Patching changes are the most dangerous. |
| Output mute states (`/main/st/mix/on`, `/bus/XX/mix/on`) | 🔴 **CRITICAL** | Can mute the PA entirely — audience gets silence. |
| Head amplifier gain (> ±12 dB change) | 🟠 **HIGH** | Large gain jumps can cause instant feedback or deafening volume. |
| EQ changes (bypass or dramatic gain) | 🟠 **HIGH** | Removing a feedback notch can re-introduce feedback. |
| Fader level changes (> 10 dB) | 🟡 **MODERATE** | Noticeable volume changes but not immediately dangerous. |
| Compressor/gate settings | 🟢 **LOW** | Affects dynamics but unlikely to cause hardware damage or feedback. |
| Scribble strip / naming changes | ⚪ **INFO** | Cosmetic only, no audio impact. |

#### Pre-flight report format
```
╔══════════════════════════════════════════════════╗
║  SCENE PRE-FLIGHT: "Sunday Contemporary"        ║
║──────────────────────────────────────────────────║
║  🔴 CRITICAL (2 issues)                          ║
║    • Main L/R routing changes from OUT 1-2       ║
║      to OUT 13-14. PA WILL GO SILENT.            ║
║    • Bus 03 [Monitor C] — currently ON, scene    ║
║      sets to OFF. Stage monitor will mute.       ║
║                                                  ║
║  🟠 HIGH (1 issue)                                ║
║    • Ch 02 [Lapel] gain: +18 dB → +42 dB         ║
║      (+24 dB jump!) — FEEDBACK RISK              ║
║                                                  ║
║  🟡 MODERATE (5 changes)                          ║
║    • Ch 01–04 fader levels change (avg ±8 dB)    ║
║                                                  ║
║  🟢 LOW (12 changes)  ⚪ INFO (8 changes)          ║
║──────────────────────────────────────────────────║
║  [L]oad anyway  [S]afe-load (skip critical)      ║
║  [R]eview details  [C]ancel                      ║
╚══════════════════════════════════════════════════╝
```

#### "Safe load" option
If the operator chooses **Safe Load**, the system loads the scene but **skips all CRITICAL
and HIGH-risk parameters**, applying only the moderate and low-risk changes. This lets a
volunteer recall a scene's EQ, compression, and naming without accidentally re-patching the PA.

### X32 OSC paths involved
- `/‐scene/N/*` — scene recall and scene data
- `/routing/*` — all routing configuration
- `/ch/XX/config/source` — channel input patching
- `/main/st/mix/*`, `/bus/XX/mix/*` — output states
- All `/ch/XX/preamp/*`, `/ch/XX/eq/*`, `/ch/XX/dyn/*`, `/ch/XX/gate/*` parameters

### Safety guardrails
- **Never auto-load**: Scenes are never loaded without the pre-flight report being presented first.
- **Safe-load default**: The default option in the UI is "Safe Load" (skip critical), not "Load anyway."
- **Undo**: Store a snapshot of the pre-load state so the scene load can be fully reverted.
- **Lock critical routing**: Optionally allow an admin/sound engineer to "lock" certain parameters (e.g., main L/R routing) so they are *always* skipped during scene recall, regardless of what the scene file contains.

### Real-world use case
> A volunteer at a conference venue needs to switch from the "Panel Discussion" scene to the
> "Keynote" scene between sessions. They trigger the scene recall. The pre-flight checker
> warns: *"🔴 CRITICAL: Main output routing will change from AES50-A 1-2 to Card 1-2.
> Your current PA is connected to AES50. PA WILL GO SILENT."* The volunteer sees this
> was a misconfigured scene from a previous event, cancels the load, and avoids 5 minutes
> of silence in front of 500 attendees.

---

## 7. Simplified TUI Dashboard (Volunteer Mode)

### Why this matters (plain language)
The X32 has 32 input channels, 16 mix buses, 6 matrices, 8 DCA groups, and hundreds of
parameters per channel. This is overwhelming for a volunteer. "Volunteer Mode" presents
a stripped-down terminal interface showing only what matters: a few big faders, mute buttons,
and clear colour-coded status indicators. Think of it as the difference between a car's
dashboard (speedo, fuel, temp) and the engine ECU diagnostic readout — volunteers get the
dashboard.

### Design principles
1. **Minimal cognitive load**: Show no more than 8–12 items on screen at once.
2. **Large touch targets**: Even on a laptop, fader elements should be large enough to operate under stage lighting with sweaty hands.
3. **Traffic-light status**: 🟢 = good, 🟡 = caution, 🔴 = problem. No numbers needed for status at a glance.
4. **Descriptive labels**: Use scribble strip names, not channel numbers. "Pastor" not "Ch 05".
5. **Progressive disclosure**: Advanced controls are available but hidden by default behind a [+] expander or an admin password.

### Display layout

```
╔════════════════════════════════════════════════════════════════╗
║  🎛️  SOUND DESK — Volunteer Mode              🟢 ALL OK      ║
╠════════════════════════════════════════════════════════════════╣
║                                                                ║
║  Pastor        Worship Ld    Lectern       Band (DCA)          ║
║  ████████▓░    ██████░░░░    ████████░░    ██████████           ║
║  ▓▓▓▓▓▓▓▓░░   ▓▓▓▓▓▓░░░░   ▓▓▓▓▓▓▓▓░░   ▓▓▓▓▓▓▓▓▓▓          ║
║  🟢 −12 dB     🟡 −22 dB     🟢 −14 dB     🟢 −10 dB           ║
║  [LIVE]        [LIVE]        [MUTED]       [LIVE]              ║
║                                                                ║
║  Playback      Choir (DCA)                                     ║
║  ████░░░░░░    ██████████                                      ║
║  ▓▓▓▓░░░░░░   ▓▓▓▓▓▓▓▓▓▓                                     ║
║  🟢 −18 dB     🟢 −8 dB                                        ║
║  [LIVE]        [LIVE]                                          ║
║                                                                ║
╠════════════════════════════════════════════════════════════════╣
║  🔴 ALERTS                                                     ║
║  • Lectern mic is MUTED — unmute when reader approaches        ║
║  • Battery warning: Worship Ld wireless < 30 min remaining     ║
╠════════════════════════════════════════════════════════════════╣
║  [M]ute all  [S]peech mode  [P]anic  [+] Advanced  [?] Help   ║
╚════════════════════════════════════════════════════════════════╝
```

### Feature breakdown

| Element | Detail |
|---|---|
| **Fader bars** | Vertical or horizontal bars showing relative level. Colour transitions from green (safe) → amber (loud) → red (clipping). Mapped from `/ch/XX/mix/fader` and `/dca/N/fader`. |
| **Level meters** | Real-time RMS/peak display from `/meters`. Show the top few dB numerically for operators learning to read levels. |
| **Mute buttons** | Large toggle. Colour: green = live, red = muted. Reads/writes `/ch/XX/mix/on`. |
| **Status badges** | 🟢🟡🔴 based on: signal present & in range (green), signal low or near clip (amber), clipping or no signal for >10s on a live channel (red). |
| **Alert panel** | Contextual messages generated by the system: muted mics that should be open, low batteries (if wireless management data available), feedback events, gain issues. |
| **DCA grouping** | Allows admin to pre-configure which channels appear as a single group fader (e.g., "Band" = channels 9–16 controlled via DCA 1). Volunteers see one fader, not eight. |
| **Quick actions** | One-key shortcuts for Speech Mode (§4), Panic (§5), and a help overlay that explains each element in plain English. |

### Admin/engineer pre-configuration
Before handing the system to a volunteer, an experienced engineer sets up:
1. Which channels/DCAs appear in Volunteer Mode (stored in a JSON config file).
2. Which features are available (e.g., enable/disable Scene recall, enable/disable advanced EQ access).
3. Alert thresholds (e.g., what level is "too quiet", what level is "too loud").
4. An optional "admin password" to unlock the full interface for troubleshooting.

### X32 OSC paths involved
- `/ch/XX/mix/fader`, `/ch/XX/mix/on` — channel levels and mutes
- `/dca/N/fader`, `/dca/N/on` — DCA group levels and mutes
- `/ch/XX/config/name`, `/ch/XX/config/icon`, `/ch/XX/config/color` — display metadata
- `/meters` — real-time metering data
- `/main/st/mix/fader`, `/main/st/mix/on` — main output level and mute
- All paths from §1–§6 features integrated via quick-action buttons

### Safety guardrails
- **No accidental damage**: Volunteer Mode hides all routing, patching, and EQ parameters by default. A volunteer cannot accidentally re-patch the PA or remove feedback notches.
- **Maximum fader limit**: Optionally cap the maximum fader value a volunteer can set (e.g., faders cannot go above −3 dB to prevent the PA from being pushed to dangerous levels).
- **Confirmation for destructive actions**: Muting the main output, recalling a scene, or changing routing requires a confirmation prompt.
- **Persistent logging**: All volunteer actions are logged with timestamps for post-event review by the admin/engineer.

### Real-world use case
> A 15-year-old volunteer is running sound for their church's youth group. They see 6 large
> faders on screen: "Speaker", "Worship Leader", "Guitar", "Keys", "Backing Track", and
> "Room Mic". Each fader has a clear green/amber/red meter and a large mute toggle. When
> the speaker finishes, the volunteer taps "MUTED" on the speaker channel and unmutes the
> worship leader. An alert pops up: *"🟡 Guitar level is low — consider raising fader."*
> They nudge it up. The entire evening runs smoothly without touching a single EQ knob.