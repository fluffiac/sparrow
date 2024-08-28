<script lang="ts">
    import { getContext, type Snippet } from "svelte";
    import type { Writable } from "svelte/store";

    let { children }: { children: Snippet } = $props();

    let random = Math.random() * 100000000;

    let list = getContext<Writable<Record<number, Snippet>>>("list");;

    list.update(list => {
        list[random] = children;
        return list
    });

    $effect(mount);
    function mount() {
        return function unmount() {
            list.update(list => {
                delete list[random];
                return list
            });
        }
    }
</script>