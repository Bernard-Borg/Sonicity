import { persisted } from "svelte-local-storage-store";

export interface Sound {
    uuid: string;
    path: string;
    keybind?: string;
};

export interface Preferences {
    darkmode: boolean;
    sequential: boolean;
    sounds: Sound[];
}

export const preferences = persisted<Preferences>('preferences', {
    darkmode: true,
    sequential: true,
    sounds: []
});