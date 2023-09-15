/// <reference types="svelte" />
/// <reference types="vite/client" />

declare module "svelte-inline-svg" {
    import { SvelteComponentTyped } from "svelte";

    export declare class InlineSvgProps {
        src: string
        transformSrc?: (src: SVGElement) => SVGElement;
        [attribute: string]: any
    }

    export default class InlineSvg extends SvelteComponentTyped<InlineSvgProps> {}
}