// Utility method to check if arrays are equal
const arraysEqual = (a, b) => {
    if (a === b) return true;
    if (a == null || b == null) return false;
    if (a.length !== b.length) return false;

    for (var i = 0; i < a.length; ++i) {
        if (a[i] !== b[i]) return false;
    }
    return true;
};

// Gets the keybind representation for display
const keybindText = (keyParts: string[]): string => {
    return keyParts.join("+");
};

// Splits the keybind parts to get a string representation of keybind parts (KeyG~~KeyA -> [KeyG, KeyA])
const deserialiseKeybind = (keybind: string): Array<string> => {
    return keybind.split("~~");
};

// Converts keybind parts (for example ["KeyG", "KeyA"]) to string for config (KeyG~~KeyA)
const serialiseKeybind = (keybindParts: Array<string>): string => {
    return keybindParts.join("~~");
};

export { deserialiseKeybind, serialiseKeybind, keybindText, arraysEqual };
