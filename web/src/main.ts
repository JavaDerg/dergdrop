import 'virtual:windi.css';
import 'virtual:windi-devtools'
import './app.css';
import App from './App.svelte';

const app = new App({
  target: document.getElementById('app') as Element,
});

export default app;
