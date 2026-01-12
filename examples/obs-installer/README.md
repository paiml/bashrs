# OBS Studio Installer for Lambda Labs Workstations

POSIX-compliant, deterministic, idempotent installer for OBS Studio optimized for NVIDIA RTX GPUs.

## Problem

The snap version of OBS Studio cannot access NVIDIA drivers due to sandbox restrictions:

```
libEGL warning: egl: failed to create dri2 screen
MESA: error: ZINK: vkCreateInstance failed (VK_ERROR_INCOMPATIBLE_DRIVER)
```

This results in:
- Software rendering instead of GPU acceleration
- No NVENC hardware encoding
- Poor performance on high-end workstations

## Solution

This installer:

1. **Removes snap OBS** - Incompatible with NVIDIA driver sandboxing
2. **Installs from official PPA** - Direct access to system NVIDIA drivers
3. **Auto-detects GPU** - Configures optimal encoder settings per GPU generation
4. **Creates optimized profile** - Pre-configured for high-quality recording

## Hardware Support

| GPU Series | Encoder | Preset | Default Bitrate |
|------------|---------|--------|-----------------|
| RTX 40xx (Ada) | NVENC H.264/HEVC | p4 | 50 Mbps |
| RTX 30xx (Ampere) | NVENC H.264 | p5 | 40 Mbps |
| Other/None | x264 (software) | veryfast | 20 Mbps |

## Usage

```bash
# Run installer
./install.sh

# Custom profile name
PROFILE_NAME="MyProfile" ./install.sh
```

## What Gets Configured

### Video Settings
- **Resolution**: 2560x1440 (matches typical Lambda workstation monitors)
- **FPS**: 60
- **Color Format**: NV12
- **Color Space**: Rec. 709

### Recording Settings (CQP Mode)
- **Encoder**: NVENC (hardware)
- **Quality**: CQP 18 (visually lossless)
- **Container**: MKV (crash-safe)

### Streaming Settings (CBR Mode)
- **Encoder**: NVENC (hardware)
- **Rate Control**: CBR
- **Bitrate**: 50 Mbps (RTX 40xx)

### Default Scene
- Screen capture (PipeWire)
- Desktop audio
- Microphone input

## File Locations

```
~/.config/obs-studio/
  global.ini                           # Global settings
  basic/
    profiles/Lambda-RTX4090/
      basic.ini                        # Video settings
      streamEncoder.json               # Streaming encoder
      recordEncoder.json               # Recording encoder
    scenes/
      Lambda-Workstation.json          # Default scene
```

## Purification Features

This installer follows Rash purified script patterns:

| Feature | Implementation |
|---------|----------------|
| **POSIX Compliant** | `#!/bin/sh` - works on dash, ash, bash |
| **Deterministic** | No `$$`, `$RANDOM`, or timestamps |
| **Idempotent** | `mkdir -p`, safe re-runs |
| **Variables Quoted** | All variables properly quoted |
| **Error Handling** | `set -euf`, explicit error checks |
| **No Network for Version** | Uses PPA latest, no API calls |

## Troubleshooting

### NVENC Not Available

Check NVIDIA driver:
```bash
nvidia-smi
```

Check OBS encoder list:
```bash
obs --help 2>&1 | grep -i encoder
```

### Screen Capture Not Working

Ensure PipeWire is running:
```bash
systemctl --user status pipewire
```

### Profile Not Loading

Verify config files:
```bash
ls -la ~/.config/obs-studio/basic/profiles/
```

## Uninstall

```bash
# Remove OBS
sudo apt remove obs-studio

# Remove PPA
sudo add-apt-repository --remove ppa:obsproject/obs-studio

# Remove config (optional)
rm -rf ~/.config/obs-studio
```
