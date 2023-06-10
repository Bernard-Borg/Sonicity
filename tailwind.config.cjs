/** @type {import('tailwindcss').Config} */
module.exports = {
    darkMode: ["class", "html.dark-theme"],
    content: ["./src/**/*.{html,js,svelte,ts}"],
    theme: {
        extend: {
            colors: {
                dark: "#171717"
            }
        }
    },
    plugins: []
};
