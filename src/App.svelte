<script lang="ts">
    import { deserialiseKeybind, serialiseKeybind, keybindText, arraysEqual } from "./utils";
    import { open } from "@tauri-apps/api/dialog";
    import { exists, writeTextFile, readTextFile, BaseDirectory, createDir } from "@tauri-apps/api/fs";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import { onMount } from "svelte";
    import { v4 as uuidv4 } from "uuid";
    import InlineSVG from "svelte-inline-svg";

    type Sound = {
        uuid: string;
        path: string;
        keybind?: string;
    };

    type SoundboardConfiguration = {
        darkmode: boolean;
        sequential: boolean;
        sounds: Array<Sound>;
    };

    // List of currently playing audio for chaos mode
    let currentAudio = [];

    // List of currently registered keybinds
    let registeredKeybinds = [];

    // Currently held keys
    let keysPressed = {};

    // Playing soundboard is true when not editing keybinds
    let playingSoundboard = true;

    let settingKeybind = "";

    let configuration: SoundboardConfiguration = {
        sounds: [],
        darkmode: true,
        sequential: true
    };

    // Loads configuration from file
    const loadConfiguration = async () => {
        if (!(await exists("soundboard-app-tauri\\soundboard-app-config.json", { dir: BaseDirectory.AppData }))) {
            await createDir("soundboard-app-tauri", { dir: BaseDirectory.AppData, recursive: true });
            writeTextFile("soundboard-app-tauri\\soundboard-app-config.json", JSON.stringify(configuration, null, 4), {
                dir: BaseDirectory.AppData
            });
        } else {
            let config = await readTextFile("soundboard-app-tauri\\soundboard-app-config.json", {
                dir: BaseDirectory.AppData
            });

            configuration = JSON.parse(config);
        }
    };

    // Rewrites the configuration to file
    const saveConfiguration = async () => {
        if (!(await exists("soundboard-app-tauri", { dir: BaseDirectory.AppData }))) {
            await createDir("soundboard-app-tauri", { dir: BaseDirectory.AppData, recursive: true });
        }

        writeTextFile("soundboard-app-tauri\\soundboard-app-config.json", JSON.stringify(configuration, null, 4), {
            dir: BaseDirectory.AppData
        });
    };

    // Theme management
    const changeTheme = async () => {
        let documentElement = document.documentElement;

        if (configuration.darkmode) {
            documentElement.classList.remove("dark-theme");
            documentElement.classList.add("light-theme");
        } else {
            documentElement.classList.remove("light-theme");
            documentElement.classList.add("dark-theme");
        }

        configuration.darkmode = !configuration.darkmode;
        saveConfiguration();
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
                configuration.sounds.push({ uuid: uuid, path: soundPath });
                configuration = configuration;

                saveConfiguration();
            }
        });
    };

    // Deletes a sound from the config
    const deleteSound = async (uuid: string): Promise<void> => {
        let indexOfItemToDelete = configuration.sounds.findIndex((x) => x.uuid == uuid);

        if (indexOfItemToDelete > -1) {
            configuration.sounds.splice(indexOfItemToDelete, 1);
            configuration = configuration;

            saveConfiguration();
        }
    };

    // //Check whether a keybind has been pressed
    // function checkKeybind(event: KeyboardEvent): void {
    //     for (let i = 0; i < registeredKeybinds.length; i++) {
    //         let containsControl = getKeybindParts(registeredKeybinds[i].keybind).includes("Ctrl");

    //         // If the keybind contains Ctrl but control is not pressed, or Ctrl is pressed but the keybind doesn't contain Ctrl then skip this
    //         if ((event.ctrlKey && !containsControl) || (!event.ctrlKey && containsControl)) {
    //             continue;
    //         }

    //         if (getKeybindText(null, getKeybindNoCtrl(registeredKeybinds[i].keybind)) == event.key) {
    //             playSound(registeredKeybinds[i].path);

    //             // Can't have multiple sounds bound to the same keybind - remove to allow
    //             break;
    //         }
    //     }
    // }

    //Plays a sound with the specified path
    function playSound(soundPath: string): void {
        soundPath = convertFileSrc(soundPath);

        if (configuration.sequential) {
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

    // Loads sound config and draws all the buttons
    async function loadPage(): Promise<void> {
        await loadConfiguration();

        document.documentElement.classList.add(configuration.darkmode ? "dark-theme" : "light-theme");
    }

    $: registeredKeybinds = configuration.sounds
        .filter((x) => x.keybind !== undefined && x.keybind !== null)
        .map((x) => ({
            uuid: x.uuid,
            path: x.path,
            keybind: deserialiseKeybind(x.keybind)
        }));

    onMount(async () => {
        loadPage();
    });

    // Debouncing mechanism
    let val = [];
    let timer;

    const debounce = (v) => {
        if (settingKeybind) {
            clearTimeout(timer);
            timer = setTimeout(() => {
                val = v;
            }, 150);
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
                alert("Keybind already set");
                return false;
            } else {
                let indexOfItemToChange = configuration.sounds.findIndex((x) => x.uuid == settingKeybind);
                configuration.sounds[indexOfItemToChange].keybind = serialiseKeybind(newKeybindKeys);
                configuration = configuration;

                saveConfiguration();
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

    $: debounce(Object.keys(keysPressed).filter((x) => keysPressed[x]));
    $: handleKeyPresses(val);
</script>

<svelte:window
    on:keydown={(e) => {
        if (!forbiddenKeybindKeys.includes(e.code)) {
            keysPressed[e.code] = true;
        } else {
            if (settingKeybind) {
                if (e.code === "Escape") {
                    // do escape stuff
                    stopRecordingKeybind();
                } else if (e.code === "Enter" || e.code == "NumpadEnter") {
                    // save keybind
                    if (tempHeldKeys.length) {
                        if (saveKeybind()) {
                            stopRecordingKeybind();
                        }
                    }
                }
            }
        }
    }}
    on:keyup={(e) => {
        if (e.code in keysPressed) {
            keysPressed[e.code] = false;
        }
    }}
/>

<main class="w-full h-screen bg-blue-100 dark:bg-dark flex flex-col">
    <div class="h-[100px] flex justify-between p-8 items-center">
        <h1 class="text-dark dark:text-white text-3xl font-bold">Sound Effects</h1>
        <!-- {keybindText(Object.keys(keysPressed).filter((x) => keysPressed[x]))}
        {registeredKeybinds.map((x) => keybindText(x.keybind))} -->
        <div class="flex items-center gap-x-3">
            <button
                class="keyboard-button flex justify-center items-center"
                on:click={() => {
                    changeTheme();
                }}
            >
                {#if configuration.darkmode}
                    <InlineSVG src="sun.svg" />
                {:else}
                    <InlineSVG src="moon.svg" />
                {/if}
            </button>
        </div>
    </div>
    <div class="flex p-10 gap-8">
        {#each configuration.sounds as sound}
            {#if sound.uuid && sound.path}
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
                        <button
                            class="keybind-text bg-gray-800 text-white rounded w-full py-1 text-sm active:outline-none text-ellipsis overflow-hidden"
                            on:click={() => {
                                stopRecordingKeybind();
                                addKeybind(sound.uuid);
                            }}
                            on:keydown={(e) => {
                                if (e.key == "Enter") {
                                    e.preventDefault();
                                }
                            }}
                            title={getDisplay(sound.uuid, keybindText(tempHeldKeys), sound.keybind)}
                        >
                            {getDisplay(sound.uuid, keybindText(tempHeldKeys), sound.keybind)}
                        </button>
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

<style lang="scss">
    $button-size: 64px;
    $button-size-active: 70px;

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

        i {
            font-size: calc($button-size / 2);
            background: -webkit-radial-gradient(circle, white, #f1f2f4);
            -webkit-background-clip: text;
            background-clip: text;
            -webkit-text-fill-color: transparent;
            text-shadow: 0 0 4px white;
        }

        &:hover {
            background: radial-gradient(circle, #2d2d2d 0%, #191919 50%);
        }

        &:active {
            height: 70px;
            margin: 5px;

            i {
                font-size: calc($button-size / 2) - 1px;
            }

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
