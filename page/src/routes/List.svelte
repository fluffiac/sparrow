<!-- ZLIST element -->
<!-- requires ELEM tag to display items -->

<script lang="ts">
    import { setContext, type Snippet } from "svelte";
    import { writable, type Writable } from "svelte/store";
    import { tweened } from 'svelte/motion';
    import { cubicOut } from 'svelte/easing';

    function drop<T>(_: T) {}

    let { children, index = $bindable(0) }: { children: Snippet, index: number } = $props();

    let list = writable({} as Writable<Record<number, Snippet>>);
    setContext("list", list);

    let zscroll = tweened(0, { duration: 1000, easing: cubicOut });
    $effect(() => {
        zscroll.set(index);
    });

    function wheel(wheel_event: WheelEvent) {
        let temp_index = index - Math.sign(wheel_event.deltaY);
        if (temp_index >= 0 && temp_index < Object.keys($list).length) {
            index = temp_index;
        }
    }

    const PANES = 6;
    const FADE = (offset: number) => 1 - Math.max(Math.min(1, (-offset - PANES + 1) * 1.25), offset * 1.25);
    const ZFN = (scroll: number) => scroll * 10 + scroll * Math.abs(scroll);
</script>

<svelte:window onwheel={wheel} />

{#snippet childs()}
    {#each Object.values($list) as child, i}
        {@const offset = $zscroll - i}
        {#if Math.floor(offset) > -(PANES + 1) && Math.floor(offset) < 1}
            {@const scroll = offset > 0 ? Math.pow(offset, 2) * 10 : offset}
            <z-card-root style:--z={ZFN(scroll)} style:--fade={FADE(offset)}>
                <z-card>
                    {@render child()}
                </z-card>
            </z-card-root>
        {/if}
    {/each}
{/snippet}

{@render children()}
<div>
    {@render childs()}
</div>

<style>
    * {
        width: 100%;
        height: 100%;
        overflow: visible;
    }

    z-card-root {
        display: block;
        padding: 1.25rem;
        
        position: absolute;
        transform-origin: 50% -15% 0px;
        transform: perspective(200px) translate3d(0px, 0px, calc(var(--z) * 1px));
        z-index: round(var(--z));
    }

    z-card {
        display: block;
        padding: 1.25rem;

        border-radius: 1.5rem;
        box-shadow: 0 0 1rem rgba(0, 0, 0, 0.1);
        backdrop-filter: blur(0.5rem); 
        background-color: rgba(4, 119, 196, 0.5);
        opacity: var(--fade);
    }

    div {
        z-index: 100;
        width: 100%;
        height: 100%;
        position: relative;
    }
</style>

