import './app.css';
import { mount } from 'svelte';
import App from './App.svelte';
import './i18n/config'; // Initialize i18n before mounting app

// Svelte 5 mount API
const app = mount(App, {
  target: document.getElementById('app')!,
});

export default app;
