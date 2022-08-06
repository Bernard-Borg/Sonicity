const { ipcRenderer } = require('electron');
const { v4: uuidv4 } = require('uuid')
const fs = require('fs');
const { eventNames } = require('process');

// View mode can either be false - soundboard mode (to use the soundboard) or true - delete mode (to delete soundboard items)
let deleteModeState = false;

// Screen can either be light (false) or dark mode (true)
let darkModeState = true;

// Playing audio can either be sequential (true) or concurrent (false)
let sequentialModeState = true;

// List of currently playing audio for chaos mode
let currentAudio = [];

// List of currently registered keybinds
let registeredKeybinds = [];

// Playing soundboard is true when not editing keybinds
let playingSoundboard = true;

let tempKeybindStore = [];

let timeoutTemp = undefined;

let settingKeybind = false;

//Plays a sound with the specified path
function playSound(path) {
    if (sequentialModeState) {
        let audio = new Audio(path);
        currentAudio.forEach(sound => {
            sound.pause();
            sound.currentTime = 0;
        });
        currentAudio = [ audio ];
        audio.play();
    } else {
        let audio = new Audio(path);
        audio.play();

        audio.onended = function() {
            let index = currentAudio.indexOf(audio);
            currentAudio.splice(index, 1);
        }

        currentAudio.push(audio);
    }
}

// Splits the keybind parts to get a string representation of keybind parts (Ctrl~~A -> [Ctrl, A])
function getKeybindParts(keybind) {
    let keybindParts = keybind.split("~~");
    return keybindParts;
}

// Converts keybind parts (for example ["Control", "A"]) to string for config (Ctrl~~A)
function getKeybindTextForFile(keybindParts) {
    let keybindResult = "";

    for(let part of keybindParts) {
        if (part == "Control") {
            keybindResult += "Ctrl~~";
        } else {
            keybindResult += part;
        }
    }

    return keybindResult;
}

//Concatenate keybind parts into a text string
function getKeybindText(keybind, keybindParts = null){
    if (keybindParts == null) {
        keybindParts = getKeybindParts(keybind);
    }
    let keybindText = "";

    for (let i = 0; i < keybindParts.length; i++) {
        if (i == keybindParts.length - 1) {
            keybindText = `${keybindText}${keybindParts[i]}`
        } else {
            keybindText = `${keybindText}${keybindParts[i]}+`
        }
    }
    
    return keybindText;
}

//Get the keybind parts without Ctrl (just the button to be pressed without Ctrl)
function getKeybindNoCtrl(keybind) {
    let keybindParts = getKeybindParts(keybind);

    let ctrlLocation = keybindParts.indexOf("Ctrl");
    
    if (ctrlLocation > -1) {
        keybindParts.splice(ctrlLocation, 1);
    }

    return keybindParts;
}

//Check whether a keybind has been pressed
function checkKeybind(event) {
    for (let i = 0; i < registeredKeybinds.length; i++) {
        let containsControl = getKeybindParts(registeredKeybinds[i].keybind).includes("Ctrl")

        // If the keybind contains Ctrl but control is not pressed, or Ctrl is pressed but the keybind doesn't contain Ctrl then skip this
        if (event.ctrlKey && !containsControl || !event.ctrlKey && containsControl) {
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
function addNewSound(functionToLoadPage) {
    ipcRenderer.invoke("showDialog").then((result) => {
        let uuid = uuidv4();
        let path = result.filePaths[0];

        if (path !== undefined) {
            let config = JSON.parse(fs.readFileSync('./sounds-config.json'));
            config.sounds.push({ uuid: uuid, path: path });
    
            fs.writeFileSync("./sounds-config.json", JSON.stringify(config, null, 4));
        }
    }).then(() => {
        functionToLoadPage();
    });
}

// Deletes a sound from the config
function deleteSound(uuid) {
    let config = JSON.parse(fs.readFileSync('./sounds-config.json'));

    let indexOfItemToDelete = config.sounds.findIndex(x => x.uuid == uuid);
    config.sounds.splice(indexOfItemToDelete, 1);

    if (uuid !== undefined) {
        fs.writeFileSync("./sounds-config.json", JSON.stringify(config, null, 4));
    }
}

// Switches between soundboard and delete modes
function changeViewMode() {
    let viewModeButton = document.getElementById("view-mode");

    if (deleteModeState) {
        viewModeButton.classList.remove("view-mode-delete");
        viewModeButton.classList.add("view-mode-soundboard");
        viewModeButton.innerText = "Delete";
    } else {
        viewModeButton.classList.remove("view-mode-soundboard");
        viewModeButton.classList.add("view-mode-delete");
        viewModeButton.innerText = "Soundboard";
    }

    deleteModeState = !deleteModeState;
}

function changeTheme() {
    let documentElement = document.documentElement

    if (darkModeState) {
        documentElement.classList.remove("dark-theme");
        documentElement.classList.add("light-theme");
    } else {
        documentElement.classList.remove("light-theme");
        documentElement.classList.add("dark-theme");
    }

    darkModeState = !darkModeState;

    let themeConfig = JSON.parse(fs.readFileSync('./app-config.json'));
    themeConfig.darkmode = darkModeState;

    fs.writeFileSync("./app-config.json", JSON.stringify(themeConfig, null, 4));
}

function changePlayMode() {
    if (sequentialModeState) {
        document.getElementById("play-mode-sequential").style.display = "none";
        document.getElementById("play-mode-chaos").style.display = "block";
    } else {
        document.getElementById("play-mode-sequential").style.display = "block";
        document.getElementById("play-mode-chaos").style.display = "none";
    }

    sequentialModeState = !sequentialModeState;

    let themeConfig = JSON.parse(fs.readFileSync('./app-config.json'));
    themeConfig.sequential = sequentialModeState;

    fs.writeFileSync("./app-config.json", JSON.stringify(themeConfig, null, 4));
}

function loadAppConfig() {
    let config = JSON.parse(fs.readFileSync('./app-config.json'));

    darkModeState = config.darkmode;
    document.documentElement.classList.add(darkModeState ? "dark-theme" : "light-theme");

    sequentialModeState = config.sequential;
    
    if (sequentialModeState) {
        document.getElementById("play-mode-chaos").style.display = "none";
        document.getElementById("play-mode-sequential").style.display = "block";
    } else {
        document.getElementById("play-mode-chaos").style.display = "block";
        document.getElementById("play-mode-sequential").style.display = "none";
    }
}

function recordKeybind(event) {
    if (event.key == "Control" || event.key == "Shift") {
        if (event.key == "Control") {
            // Pressing controls resets the keybind
            if (tempKeybindStore.length > 0) {
                tempKeybindStore = [];
            }

            tempKeybindStore.push("Control");
        }
    } else {
        // Cannot have more than 1 normal character, but can have Control
        if (tempKeybindStore.length >= 1 && !tempKeybindStore.includes("Control") || tempKeybindStore.includes("Control") && tempKeybindStore.length > 1) {
            tempKeybindStore = [];
        }

        tempKeybindStore.push(event.key);
    }
}

function handleKeydownEvent(event) {
    if (playingSoundboard) {
        checkKeybind(event);
    } else {
        recordKeybind(event);   
    }
}

function isTempKeybindValid() {
    if (tempKeybindStore.includes("Control") && tempKeybindStore.length > 1 || tempKeybindStore.length == 1) {
        if (timeoutTemp !== undefined) {
            clearTimeout(timeoutTemp);
        }
        return true;
    } else {
        timeoutTemp = setTimeout(isTempKeybindValid, 50)
        return false;
    }
}

// Bind a keybind to a button
async function addKeybind(uuid) {
    playingSoundboard = false;
    settingKeybind = true;

    while(!isTempKeybindValid()) {
        await new Promise(resolve => setTimeout(resolve, 50));
    }

    let config = JSON.parse(fs.readFileSync('./sounds-config.json'));

    if (uuid !== undefined) {
        let newKeybind = getKeybindTextForFile(tempKeybindStore);

        // If keybind already set, alert the user, otherwise change the config file
        if (registeredKeybinds.map(x => x.keybind).includes(newKeybind)) {
            alert("Keybind already set");
        } else {
            let indexOfItemToChange = config.sounds.findIndex(x => x.uuid == uuid);
            config.sounds[indexOfItemToChange].keybind = newKeybind;
            fs.writeFileSync("./sounds-config.json", JSON.stringify(config, null, 4));
        }
    }

    playingSoundboard = true;
    settingKeybind = false;
    tempKeybindStore = [];
    clearTimeout(timeoutTemp);
}

// Loads sound config and draws all the buttons 
function loadPage() {
    loadAppConfig();

    // Load the sound configuration data
    const sounds_config = JSON.parse(fs.readFileSync('./sounds-config.json'));

    // Store the sound data in a variable
    let sounds = sounds_config.sounds

    // Get a list of keybinds currently set to a sound effect
    registeredKeybinds = sounds.filter(x => x.keybind !== undefined && x.keybind !== null);

    const createSoundboardButton = (uuid, path, keybind) => {
        if (uuid !== undefined && path !== undefined) {
            let container = document.createElement("div");
            container.setAttribute("class", "button-container");

            let buttonElement = document.createElement("button");
            buttonElement.setAttribute("id", uuid);
            buttonElement.setAttribute("class", "sound-button");
            
            let filename = path.replace(/^.*[\\\/]/, '')
            buttonElement.innerText = filename;

            buttonElement.addEventListener("click", () => {
                playSound(path);
            });

            let textElement = document.createElement("span");
            textElement.setAttribute("class", "keybind-text");

            // If a keybind is set, use that as the keybind text, otherwise use the filename
            if (keybind !== undefined) {
                let keybindText = getKeybindText(keybind);
                textElement.innerText = keybindText;
            } else {
                textElement.innerText = "Add keybind";
            }
            
            textElement.addEventListener("click", async () => {
                if (!settingKeybind) {
                    textElement.textContent = "...";
                    await addKeybind(uuid);
                    loadPage();
                }
            });

            container.appendChild(buttonElement);
            container.appendChild(textElement);

            return container;
        } else {
            console.debug(`Attempted to create soundboard button with UUID ${uuid ?? 'empty'} and Path ${path ?? 'empty'}`)
            return;
        }
    }

    const createDeleteButton = (uuid, path, keybind) => {
        if (uuid !== undefined && path !== undefined) {
            let element = document.createElement("button");
            element.setAttribute("id", uuid);
            element.setAttribute("class", "sound-button sound-button-delete")

            let filename = path.replace(/^.*[\\\/]/, '')
            element.innerText = filename;

            element.addEventListener("click", () => {
                deleteSound(uuid);
                loadPage();
            });

            return element;
        } else {
            console.debug(`Attempted to create delete button with UUID ${uuid ?? 'empty'} and Path ${path ?? 'empty'}`)
            return;
        }
    }

    let buttonArray = []

    for (const type of sounds) {
        if (!deleteModeState) {
            let button = createSoundboardButton(type.uuid, type.path, type.keybind);

            if (button !== undefined) {
                buttonArray.push(button);
            }
        } else {
            let button = createDeleteButton(type.uuid, type.path, type.keybind);

            if (button !== undefined) {
                buttonArray.push(button);
            }
        }
    }

    let mainDiv = document.getElementById('main-container');

    if (!deleteModeState) {
        let element = document.createElement("button");

        element.setAttribute("id", "add-new-sound");
        element.innerText = "+";

        element.addEventListener("click", () => {
            addNewSound(loadPage);
        });

        buttonArray.push(element);
    }

    mainDiv.replaceChildren(...buttonArray);
}

// Render the soundboard buttons
loadPage();

let viewModeButton = document.getElementById("view-mode");

//Register event listeners
viewModeButton.addEventListener("click", () => {
    changeViewMode();
    loadPage();
});

let themeButton = document.getElementById("dark-mode");

themeButton.addEventListener("click", () => {
    changeTheme();
});

let playModeButton = document.getElementById("play-mode");

playModeButton.addEventListener("click", () => {
    changePlayMode();
});

window.addEventListener("keydown", handleKeydownEvent);