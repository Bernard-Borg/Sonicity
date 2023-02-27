<script lang="ts">
    import { open } from "@tauri-apps/api/dialog";
    import { exists, writeTextFile, readTextFile, BaseDirectory, createDir } from "@tauri-apps/api/fs";
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

    // View mode can either be false - soundboard mode (to use the soundboard) or true - delete mode (to delete soundboard items)
    let deleteMode = false;

    // List of currently playing audio for chaos mode
    let currentAudio = [];

    // List of currently registered keybinds
    let registeredKeybinds = [];

    // Playing soundboard is true when not editing keybinds
    let playingSoundboard = true;

    let tempKeybindStore = [];

    let timeoutTemp = undefined;

    let settingKeybind = "";

    let configuration: SoundboardConfiguration = {
        sounds: [],
        darkmode: true,
        sequential: false
    };

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

    // Splits the keybind parts to get a string representation of keybind parts (Ctrl~~A -> [Ctrl, A])
    function getKeybindParts(keybind: string): Array<string> {
        let keybindParts = keybind.split("~~");
        return keybindParts;
    }

    // Converts keybind parts (for example ["Control", "A"]) to string for config (Ctrl~~A)
    function getKeybindTextForFile(keybindParts: Array<string>): string {
        let keybindResult = "";

        for (let part of keybindParts) {
            if (part == "Control") {
                keybindResult += "Ctrl~~";
            } else {
                keybindResult += part;
            }
        }

        return keybindResult;
    }

    //Concatenate keybind parts into a text string
    function getKeybindText(keybind: string, keybindParts: Array<string> = null) {
        if (keybindParts == null) {
            keybindParts = getKeybindParts(keybind);
        }
        let keybindText = "";

        for (let i = 0; i < keybindParts.length; i++) {
            if (i == keybindParts.length - 1) {
                keybindText = `${keybindText}${keybindParts[i]}`;
            } else {
                keybindText = `${keybindText}${keybindParts[i]}+`;
            }
        }

        return keybindText;
    }

    //Get the keybind parts without Ctrl (just the button to be pressed without Ctrl)
    function getKeybindNoCtrl(keybind: string) {
        let keybindParts = getKeybindParts(keybind);

        let ctrlLocation = keybindParts.indexOf("Ctrl");

        if (ctrlLocation > -1) {
            keybindParts.splice(ctrlLocation, 1);
        }

        return keybindParts;
    }

    //Check whether a keybind has been pressed
    function checkKeybind(event: KeyboardEvent) {
        for (let i = 0; i < registeredKeybinds.length; i++) {
            let containsControl = getKeybindParts(registeredKeybinds[i].keybind).includes("Ctrl");

            // If the keybind contains Ctrl but control is not pressed, or Ctrl is pressed but the keybind doesn't contain Ctrl then skip this
            if ((event.ctrlKey && !containsControl) || (!event.ctrlKey && containsControl)) {
                continue;
            }

            if (getKeybindText(null, getKeybindNoCtrl(registeredKeybinds[i].keybind)) == event.key) {
                playSound(registeredKeybinds[i].path);

                // Can't have multiple sounds bound to the same keybind - remove to allow
                break;
            }
        }
    }

    // Displays an open file dialog and adds a new sound config
    function addNewSound() {
        open({
            multiple: false,
            filters: [{ name: "Audio Files", extensions: ["wav", "ogg", "mp3"] }]
        }).then((result) => {
            let uuid = uuidv4();
            let soundPath = result[0];

            if (soundPath !== undefined) {
                configuration.sounds.push({ uuid: uuid, path: soundPath });
                configuration = configuration;

                saveConfiguration();
            }
        });
    }

    // Deletes a sound from the config
    async function deleteSound(uuid) {
        let indexOfItemToDelete = configuration.sounds.findIndex((x) => x.uuid == uuid);

        if (indexOfItemToDelete > -1) {
            configuration.sounds.splice(indexOfItemToDelete, 1);
            configuration = configuration;

            saveConfiguration();
        }
    }

    // Switches between soundboard and delete modes
    function changeViewMode() {
        deleteMode = !deleteMode;
    }

    async function changeTheme() {
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
    }

    async function changePlayMode() {
        if (configuration.sequential) {
            document.getElementById("play-mode-sequential").style.display = "none";
            document.getElementById("play-mode-chaos").style.display = "block";
        } else {
            document.getElementById("play-mode-sequential").style.display = "block";
            document.getElementById("play-mode-chaos").style.display = "none";
        }

        configuration.sequential = !configuration.sequential;

        saveConfiguration();
    }

    //Plays a sound with the specified path
    function playSound(soundPath: string) {
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

    function recordKeybind(event: KeyboardEvent) {
        if (event.key == "Control" || event.key == "Shift") {
            if (event.key == "Control") {
                // Pressing controls resets the keybind
                if (tempKeybindStore.length > 0) {
                    tempKeybindStore = [];
                }

                tempKeybindStore.push("Control");
                tempKeybindStore = tempKeybindStore;
            }
        } else {
            // Cannot have more than 1 normal character, but can have Control
            if (
                (tempKeybindStore.length >= 1 && !tempKeybindStore.includes("Control")) ||
                (tempKeybindStore.includes("Control") && tempKeybindStore.length > 1)
            ) {
                tempKeybindStore = [];
            }

            tempKeybindStore.push(event.key);
            tempKeybindStore = tempKeybindStore;
        }
    }

    function handleKeydownEvent(event: KeyboardEvent) {
        if (playingSoundboard) {
            checkKeybind(event);
        } else {
            recordKeybind(event);
        }
    }

    function isTempKeybindValid() {
        if ((tempKeybindStore.includes("Control") && tempKeybindStore.length > 1) || tempKeybindStore.length == 1) {
            if (timeoutTemp !== undefined) {
                clearTimeout(timeoutTemp);
            }
            return true;
        } else {
            timeoutTemp = setTimeout(isTempKeybindValid, 50);
            return false;
        }
    }

    // Bind a keybind to a button
    async function addKeybind(uuid) {
        playingSoundboard = false;
        settingKeybind = uuid;

        while (!isTempKeybindValid()) {
            await new Promise((resolve) => setTimeout(resolve, 50));
        }

        if (uuid !== undefined) {
            let newKeybind = getKeybindTextForFile(tempKeybindStore);

            // If keybind already set, alert the user, otherwise change the config file
            if (registeredKeybinds.map((x) => x.keybind).includes(newKeybind)) {
                alert("Keybind already set");
            } else {
                let indexOfItemToChange = configuration.sounds.findIndex((x) => x.uuid == uuid);
                configuration.sounds[indexOfItemToChange].keybind = newKeybind;
                configuration = configuration;

                saveConfiguration();
            }
        }

        playingSoundboard = true;
        settingKeybind = "";
        tempKeybindStore = [];
        clearTimeout(timeoutTemp);
    }

    // Loads sound config and draws all the buttons
    async function loadPage() {
        await loadConfiguration();

        document.documentElement.classList.add(configuration.darkmode ? "dark-theme" : "light-theme");

        // Get a list of keybinds currently set to a sound effect
        registeredKeybinds = configuration.sounds.filter((x) => x.keybind !== undefined && x.keybind !== null);
    }

    onMount(async () => {
        loadPage();
    });
</script>

<svelte:window
    on:keydown={(e) => {
        handleKeydownEvent(e);
    }}
/>

<main class="w-full h-screen bg-zinc-700 dark:bg-dark flex flex-col">
    <div class="h-[100px] flex justify-between p-8 items-center">
        <h1 class="text-dark dark:text-white text-3xl font-bold">Sound Effects</h1>
        <div class="flex items-center gap-x-3">
            <button
                class="keyboard-button"
                on:click={() => {
                    changePlayMode();
                }}
            >
                Play Mode
            </button>
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
            <button
                class="keyboard-button flex justify-center items-center"
                on:click={() => {
                    changeViewMode();
                    loadPage();
                }}
            >
                {#if deleteMode}
                    <InlineSVG src="corner-up-left.svg" />
                {:else}
                    <InlineSVG src="trash-2.svg" />
                {/if}
            </button>
        </div>
    </div>
    <div class="flex py-10 gap-3">
        {#each configuration.sounds as sound}
            {#if !deleteMode && sound.uuid !== undefined && sound.path !== undefined}
                <div class="flex flex-col">
                    <button
                        id={sound.uuid}
                        class="w-[100px] p-3 text-ellipsis whitespace-nowrap overflow-hidden aspect-square border border-solid border-black rounded-full dark:border-white"
                        on:click={() => {
                            playSound(sound.path);
                        }}>{sound.path.replace(/^.*[\\\/]/, "")}</button
                    >
                    <!-- svelte-ignore a11y-click-events-have-key-events -->
                    <span
                        class="keybind-text"
                        on:click={async () => {
                            if (!settingKeybind) {
                                await addKeybind(sound.uuid);
                            }
                        }}
                        >{settingKeybind === sound.uuid
                            ? "..."
                            : sound.keybind !== undefined
                            ? getKeybindText(sound.keybind)
                            : "Add keybind"}</span
                    >
                </div>
            {:else if deleteMode && sound.uuid !== undefined && sound.path !== undefined}
                <button
                    id={sound.uuid}
                    class="w-[100px] p-3 text-ellipsis whitespace-nowrap overflow-hidden aspect-square border border-solid border-black rounded-full dark:border-white"
                    on:click={() => {
                        deleteSound(sound.uuid);
                    }}>{sound.path.replace(/^.*[\\\/]/, "")}</button
                >
            {/if}
        {/each}
        {#if !deleteMode}
            <button
                class="w-[100px] h-[100px] p-3 text-ellipsis whitespace-nowrap overflow-hidden border border-black border-solid rounded-full dark:border-white"
                on:click={() => {
                    addNewSound();
                }}>+</button
            >
        {/if}
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
