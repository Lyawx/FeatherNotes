import { createApp } from 'vue';
import App from './App.vue';
import './css/markdown.css';
import './css/variables.css'
import { ThemeService } from './services/themeService';

async function bootstrap() {
  // Charge le thème utilisateur depuis les configurations avant le montage de l'UI
  await ThemeService.init();

  const app = createApp(App);
  app.mount('#app');
}

bootstrap();