<script lang="ts">
import { deserialiseKeybind, serialiseKeybind, keybindText, arraysEqual } from "./utils";
import { open } from "@tauri-apps/api/dialog";
import { convertFileSrc } from "@tauri-apps/api/tauri";
import { onDestroy, onMount } from "svelte";
import { v4 as uuidv4 } from "uuid";
import InlineSVG from "svelte-inline-svg";
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { preferences } from "./stores/preferences";

type Payload = {
    key_pressed: string;
}

// List of currently playing audio for chaos mode
let currentAudio = [];

// Currently held keys
let keysPressed = {};

// Playing soundboard is true when not editing keybinds
let playingSoundboard = true;

let settingKeybind = "";

let unlisten: UnlistenFn;
let unlisten2: UnlistenFn;

// Theme management
const changeTheme = async () => {
    preferences.update((preferences) => {
        preferences.darkmode = !preferences.darkmode;
        return preferences;
    });
};

// Playmode management
const changePlaymode = async () => {
    preferences.update((preferences) => {
        preferences.sequential = !preferences.sequential;
        return preferences;
    });
};

// Displays an open file dialog and adds a new sound config
const addNewSound = async (): Promise<void> => {
    open({
        multiple: false,
        filters: [{ name: "Audio Files", extensions: ["wav", "ogg", "mp3"] }]
    }).then((result) => {
        let uuid = uuidv4();
        let soundPath = "";

        if (Array.isArray(result)) {
            soundPath = result[0];
        } else {
            soundPath = result;
        }

        if (soundPath !== undefined) {
            preferences.update((preferences) => {
                preferences.sounds.push({ uuid: uuid, path: soundPath });
                return preferences
            });
        }
    });
};

// Deletes a sound from the config
const deleteSound = async (uuid: string): Promise<void> => {
    let indexOfItemToDelete = $preferences.sounds.findIndex((x) => x.uuid == uuid);

    if (indexOfItemToDelete > -1) {
        preferences.update((preferences) => {
            preferences.sounds.splice(indexOfItemToDelete, 1);
            return preferences
        });
    }
};

//Plays a sound with the specified path
function playSound(soundPath: string): void {
    soundPath = convertFileSrc(soundPath);

    if ($preferences.sequential) {
        let audio = new Audio(soundPath);
        
        currentAudio.forEach((sound) => {
            sound.pause();
            sound.currentTime = 0;
        });

        currentAudio = [audio];

        audio.play();
    } else {
        let audio = new Audio(soundPath);
        audio.play();

        audio.onended = function () {
            let index = currentAudio.indexOf(audio);
            currentAudio.splice(index, 1);
        };

        currentAudio.push(audio);
    }
}

const addKeybind = async (uuid: string): Promise<void> => {
    playingSoundboard = false;
    settingKeybind = uuid;
};

$: registeredKeybinds = $preferences.sounds
    .filter((x) => x.keybind !== undefined && x.keybind !== null)
    .map((x) => ({
        uuid: x.uuid,
        path: x.path,
        keybind: deserialiseKeybind(x.keybind)
    }));

onMount(async () => {
    unlisten = await listen('keypress', (event) => {
        let pressedKey = (event.payload as Payload).key_pressed;

        if (!forbiddenKeybindKeys.includes(pressedKey)) {
            keysPressed[pressedKey] = true;
        } else {
            if (settingKeybind) {
                if (pressedKey === "Escape") {
                    // do escape stuff
                    stopRecordingKeybind();
                } else if (pressedKey === "Enter" || pressedKey == "NumpadEnter") {
                    // save keybind
                    if (tempHeldKeys.length) {
                        if (saveKeybind()) {
                            stopRecordingKeybind();
                        }
                    }
                }
            }
        }
    })

    unlisten2 = await listen('keyup', (event) => {
        let pressedKey = (event.payload as Payload).key_pressed;

        if (pressedKey in keysPressed) {
            keysPressed[pressedKey] = false;
        }
    })
});

onDestroy(() => {
    if (unlisten) {
        unlisten();
    }

    if (unlisten2) {
        unlisten2();
    }
})

// Debouncing mechanism
let val = [];
let timer;

const debounce = (v) => {
    if (settingKeybind) {
        clearTimeout(timer);
        timer = setTimeout(() => {
            val = v;
        }, 10);
    } else {
        val = v;
    }
};

let tempHeldKeys = [];

const handleKeyPresses = (heldKeys: string[]) => {
    if (!heldKeys.length) {
        return;
    }

    if (settingKeybind) {
        tempHeldKeys = heldKeys;
    } else {
        let keybindExists = registeredKeybinds.filter(
            (x) => x.keybind && arraysEqual(x.keybind.sort(), heldKeys.sort())
        );

        if (keybindExists.length) {
            playSound(keybindExists[0].path);
        }
    }
};

const saveKeybind = (): boolean => {
    if (settingKeybind) {
        let newKeybindKeys = tempHeldKeys.sort();

        if (
            registeredKeybinds
                .filter((x) => x.uuid !== settingKeybind)
                .map((x) => x.keybind)
                .filter((x) => x && arraysEqual(x.sort(), newKeybindKeys)).length
        ) {
            return false;
        } else {
            let indexOfItemToChange = $preferences.sounds.findIndex((x) => x.uuid == settingKeybind);

            preferences.update((preferences) => {
                preferences.sounds[indexOfItemToChange].keybind = serialiseKeybind(newKeybindKeys);
                return preferences
            });
            
            console.debug(`Saved ${serialiseKeybind(newKeybindKeys)} for ${settingKeybind}`);
            return true;
        }
    }
};

const stopRecordingKeybind = (): void => {
    settingKeybind = "";
    tempHeldKeys = [];
    playingSoundboard = true;
};

const getDisplay = (uuid: string, heldKeys: string, keybind: string): string => {
    if (settingKeybind !== uuid) {
        if (keybind) {
            return keybindText(deserialiseKeybind(keybind));
        } else {
            return "Add keybind";
        }
    } else {
        if (heldKeys) {
            return heldKeys;
        } else {
            return "...";
        }
    }
};

// The following are special keys with special functionality and therefore cannot be used for keybinds
let forbiddenKeybindKeys = ["Escape", "Enter", "NumpadEnter"];

$: debounce(Object.keys(keysPressed).filter((x) => keysPressed[x] === true));
$: handleKeyPresses(val);
</script>

<svelte:window on:click|self={() => stopRecordingKeybind()} class="{$preferences.darkmode ? 'dark-theme' : 'light-theme'}" />

<main class="w-full h-screen bg-blue-100 dark:bg-dark flex flex-col">
    <div class="h-[100px] flex justify-between p-8 items-center">
        <h1 class="text-dark dark:text-white text-3xl font-bold">Sound Effects</h1>
        <div class="flex items-center gap-x-3">
            <button
                class="keyboard-button flex justify-center items-center"
                title={$preferences.sequential ? 'Switch to sequential mode' : 'Switch to chaos mode'}
                on:click={() => {
                    changePlaymode();
                }}
            >
                {#if $preferences.sequential}
                    <InlineSVG src="stop-circle.svg" />
                {:else}
                    <InlineSVG src="chevrons-right.svg" />
                {/if}
            </button>
            <button
                class="keyboard-button flex justify-center items-center"
                title={$preferences.darkmode ? 'Switch to dark mode' : 'Switch to light mode'}
                on:click={() => {
                    changeTheme();
                }}
            >
                {#if $preferences.darkmode}
                    <InlineSVG src="sun.svg" />
                {:else}
                    <InlineSVG src="moon.svg"/>
                {/if}
            </button>
        </div>
    </div>
    <div class="flex p-10 gap-8">
        {#each $preferences.sounds as sound}
            {#if sound.uuid && sound.path}
                {@const display = getDisplay(sound.uuid, keybindText(tempHeldKeys), sound.keybind)}
                {@const newKeybindKeys = tempHeldKeys.sort()}
                {@const canRegisterKeybind = registeredKeybinds
                    .filter((x) => x.uuid !== settingKeybind)
                    .map((x) => x.keybind)
                    .filter((x) => x && arraysEqual(x.sort(), newKeybindKeys)).length <= 0}
                <button
                    id={sound.uuid}
                    class="flex flex-col justify-between w-[140px] border border-black rounded-lg h-[160px] p-2 bg-white shadow-xl cursor-pointer hover:shadow-none active:bg-gray-100"
                    on:click|self={() => {
                        playSound(sound.path);
                    }}
                >
                    <span
                        class="w-full max-h-[70px] text-ellipsis overflow-hidden active:pointer-events-none"
                        title={sound.path.replace(/^.*[\\\/]/, "")}
                    >
                        {sound.path.replace(/^.*[\\\/]/, "")}
                    </span>
                    <div class="flex w-full gap-1">
                        <div class="flex flex-col-reverse w-full overflow-hidden">
                            <button
                                class="keybind-text bg-gray-800 text-white rounded py-1 text-sm active:outline-none text-ellipsis z-20 {!canRegisterKeybind ? 'outline-red-400' : ''}"
                                on:click={() => {
                                    stopRecordingKeybind();
                                    addKeybind(sound.uuid);
                                }}
                                on:keydown={(e) => {
                                    if (e.key == "Enter") {
                                        e.preventDefault();
                                    }
                                }}
                                title={display}
                            >
                                {display}
                            </button>
                            <span class="{settingKeybind === sound.uuid ? !canRegisterKeybind ? 'show-fail' : keybindText(tempHeldKeys) ? 'show-enter' : '' : ''}"></span>
                        </div>
                        <button
                            on:click={() => {
                                deleteSound(sound.uuid);
                            }}
                        >
                            <InlineSVG src="trash-2.svg" />
                        </button>
                    </div>
                </button>
            {/if}
        {/each}
        <button
            class="w-[50px] h-[50px] p-3 text-ellipsis whitespace-nowrap overflow-hidden border border-black border-solid rounded-md bg-red-500 text-white dark:border-white"
            on:click={() => {
                addNewSound();
            }}><InlineSVG src="plus.svg" /></button
        >
    </div>
</main>

<style lang="scss" global>
$button-size: 64px;
$button-size-active: 70px;

.show-enter, .show-fail {
    transition: transform 0.2s linear;
    transform: translateY(-10px);

    &::after {
        transform: translateY(10px);
        content: "Press Enter";
        position: relative;
        font-size: 10px;
        background: white;
        color: black;
        display: block;
        border: 1px solid black;
        border-top-right-radius: 5px;
        border-top-left-radius: 5px;
    }
}

.show-fail {
    &::after {
        content: "Already used";
    }
}

.keyboard-button {
    background: radial-gradient(circle, #292929 0%, #131615 50%);
    height: $button-size;
    aspect-ratio: 1/1;
    border-radius: 100%;
    position: relative;
    z-index: 1;
    transform-style: preserve-3d;
    border: none;
    color: white;
    cursor: pointer;
    margin: calc($button-size / 8);

    svg:focus {
        outline: none;
    }

    &:hover {
        background: radial-gradient(circle, #2d2d2d 0%, #191919 50%);
    }

    &:active {
        height: 70px;
        margin: 5px;

        &::after {
            left: -5px;
            top: -5px;
            width: calc(100% + 10px);
            height: calc(100% + 10px);
        }
    }

    &::after {
        content: "";
        position: absolute;
        left: calc(-#{$button-size} / 8);
        top: calc(-#{$button-size} / 8);
        width: calc(100% + ((#{$button-size} / 8) * 2));
        height: calc(100% + ((#{$button-size} / 8) * 2));
        background: linear-gradient(to right, #050a09, #242625);
        border-radius: 100%;
        transform: translateZ(-1px);
        outline: 1px solid black;
    }
}
</style>
