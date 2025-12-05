import './app.css';
import App from './App.svelte';

// Svelte 5 mount API
const app = App({
  target: document.getElementById('app')!,
});

export default app;
