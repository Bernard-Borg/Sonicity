# SoundboardApp
A soundboard app similar to Voicemod, but free - previously created as a project to learn ElectronJS and NodeJS with. It has now been rebuilt with [Tauri](https://tauri.app/), Svelte and TailwindCSS as a way to reduce the app's size, and learn those technologies.

## Features

The interface is quite simple;

- You can add new sounds by clicking the + button.
- You can delete sounds by clicking on the trash can.
- You can set keybinds by clicking on "Add keybind" under the sound.
- You can change the playback mode (chaos mode - play multiple sounds at the same time or sequential - when playing a sound, stop all previous ones).
- You can change the app theme (light mode or dark mode).

## App screenshot

![app screenshot](https://github.com/Bernard-Borg/SoundboardApp/assets/35971384/02f34d1e-5588-45bc-aef1-5a4f68e99ac0)

## Further improvements

Currently thinking about;

- Renaming clips
- Renaming the application
- Adding an icon
- Playing sound through rust rather than JavaScript (which would allow you to switch device to play sound in, possibly supporting existing virtual microphones)

## Other notes

I have no idea how to create drivers to be able to stream the audio through a microphone, but I will try my best.
