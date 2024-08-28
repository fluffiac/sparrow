<script lang=ts>
    import List from "./List.svelte";
    import Elem from "./Elem.svelte";

    let list: number[] = $state([]);
    
    setInterval(() => {
        list = [...list, list.length];
    }, 1000);

    let index = $state(3);

    window.onkeydown = (e) => {
        if (e.key === "ArrowDown") {
            index = 0;
        } else if (e.key === "ArrowUp") {
            index = list.length;
        }
    };
</script>

<div class="container">
    <div class=sb>
        <h1>SIDE BAR !!! :)</h1>
    </div>
    <List bind:index>
        <Elem>
            <h1>Head</h1>
        </Elem>
        {#each list as i}
            <Elem>
                <h1>Page {i}</h1>
                <p>Lorem ipsum dolor sit amet consectetur, adipisicing elit. Rem cumque sunt fugit quo quia neque, officiis, quam exercitationem, ratione perspiciatis ex ullam delectus enim maxime veritatis? Alias nam quos laudantium.</p>
                <p>Lorem ipsum dolor sit amet consectetur, adipisicing elit. Rem cumque sunt fugit quo quia neque, officiis, quam exercitationem, ratione perspiciatis ex ullam delectus enim maxime veritatis? Alias nam quos laudantium.</p>
                <p>Lorem ipsum dolor sit amet consectetur, adipisicing elit. Rem cumque sunt fugit quo quia neque, officiis, quam exercitationem, ratione perspiciatis ex ullam delectus enim maxime veritatis? Alias nam quos laudantium.</p>
                <p>Lorem ipsum dolor sit amet consectetur, adipisicing elit. Rem cumque sunt fugit quo quia neque, officiis, quam exercitationem, ratione perspiciatis ex ullam delectus enim maxime veritatis? Alias nam quos laudantium.</p>
                <p>Lorem ipsum dolor sit amet consectetur, adipisicing elit. Rem cumque sunt fugit quo quia neque, officiis, quam exercitationem, ratione perspiciatis ex ullam delectus enim maxime veritatis? Alias nam quos laudantium.</p>
                <p>Lorem ipsum dolor sit amet consectetur, adipisicing elit. Rem cumque sunt fugit quo quia neque, officiis, quam exercitationem, ratione perspiciatis ex ullam delectus enim maxime veritatis? Alias nam quos laudantium.</p>
                <p>Lorem ipsum dolor sit amet consectetur, adipisicing elit. Rem cumque sunt fugit quo quia neque, officiis, quam exercitationem, ratione perspiciatis ex ullam delectus enim maxime veritatis? Alias nam quos laudantium.</p>
                <p>Lorem ipsum dolor sit amet consectetur, adipisicing elit. Rem cumque sunt fugit quo quia neque, officiis, quam exercitationem, ratione perspiciatis ex ullam delectus enim maxime veritatis? Alias nam quos laudantium.</p>
            </Elem>
        {/each}
    </List>
</div>

<style>
    .container {
        display: flex;
        flex-direction: column;
        height: 100%;
    }

    .container > :global(*) {
        flex: 1;
    }
</style>
