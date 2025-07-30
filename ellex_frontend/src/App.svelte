<script>
	import CodeMirror from "svelte-codemirror-editor";
	import Cytoscape from "cytoscape"; // Or svelte wrapper
	import { onMount } from "svelte";

	let irCode = "; LLVM-IR here";
	let graphElements = []; // { data: { id: 'node1' } }

	onMount(() => {
		// Init Cytoscape in #graph
		Cytoscape({
			container: document.getElementById("graph"),
			elements: graphElements,
		});
	});
	let apiUrl = "http://localhost:8080";

	async function parseCode() {
		const res = await fetch(`${apiUrl}/api/editor/parse`, {
			method: "POST",
			body: JSON.stringify({ code: codeValue, visualize: true }),
			headers: { "Content-Type": "application/json" },
		});
		const data = await res.json();
		output = data.output;
	}

	async function updateGraph() {
		const res = await fetch(`${apiUrl}/api/ir/visualize`, {
			method: "POST",
			body: JSON.stringify({ ir_code: irCode }),
			headers: { "Content-Type": "application/json" },
		});
		const data = await res.json();
		graphElements = JSON.parse(data.graph); // Update Cytoscape
	}
	function updateGraph() {
		// Fetch from backend or client-side parse
		graphElements = [
			{ data: { id: "entry", label: "Entry Block" } },
			{ data: { id: "loop", label: "Loop Body" } },
			{ data: { source: "entry", target: "loop", label: "br" } },
		];
	}
</script>

<div class="tabs">
	<!-- Tabs for Editor/Output/LLVM-IR -->
</div>

<div class="split">
	<div class="editor">
		<CodeMirror bind:value={irCode} lang="llvm" theme="dracula" />
	</div>
	<div id="graph" class="graph"></div>
</div>

<button on:click={updateGraph}>Update Graph</button>

<style>
	.split {
		display: flex;
	}
	.editor,
	.graph {
		width: 50%;
		height: 500px;
	}
</style>
