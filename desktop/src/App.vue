<script setup lang="ts">
import { RouterView } from 'vue-router';
import DKLogo from '@/components/DKLogo.vue';
import LangSelect from '@/views/LangSelect.vue';
import DKLayout from '@/components/DKLayout.vue';
</script>

<script lang="ts">
import { defineComponent, inject } from 'vue';

export default defineComponent({
  data() {
    return {
      page_number: 0,
      progress: 0,
      config: {},
      langSelected: false,
      current_lang: 'en',
      lightup: 0,
      timer: null,
      can_quit: true,
      curr: 0,
      switchLocale: inject('switchLocale') as (text: string) => Promise<void>,
    };
  },
  computed: {},
  methods: {
    on_abort() {
      const { path } = this.$router.currentRoute.value;
      if (path === '/finish') {
        this.$router.push({
          path: '/abort',
          query: { done: 1 },
        });
      } else {
        this.$router.push('/abort');
      }
    },
    nav_menu_bold(step: number) {
      return this.page_number >= step ? 'active' : '';
    },
    lightup_seq(step: number) {
      return this.lightup >= step ? '' : 'hidden';
    },
    execute_lightup() {
      const timer = setInterval(() => {
        if (this.lightup + 1 >= 4) clearInterval(timer);
        this.lightup += 1;
      }, 210);
    },
    on_lang_selected(id: string) {
      this.current_lang = id.toLowerCase();
      if (id === 'en') {
        // default locale is always loaded before-hand
        this.langSelected = true;
        this.execute_lightup();
        return;
      }
      // lazy load the translation strings
      this.switchLocale(id)
        .then(() => {
          this.langSelected = true;
          this.execute_lightup();
        })
        .catch(() => {
          this.langSelected = true;
          this.execute_lightup();
        });
    },
  },
  async mounted() {
    window.addEventListener('contextmenu', (event) => {
      event.preventDefault();
    });

    this.$router.beforeEach((to) => {
      if (to.name === 'error' || to.name === 'abort') return;
      this.page_number = to.meta.steps as number;
      this.progress = this.page_number * 25;
    });
  },
  provide() {
    return {
      config: {},
    };
  },
  async created() {},
});
</script>

<template>
  <div
    :class="'lang-' + current_lang"
    style="padding: 0 2rem; margin-bottom: 1rem"
  >
    <header style="width: 90%" v-if="langSelected">
      <DKLogo />
    </header>
  </div>
  <!-- language select overlay -->
  <LangSelect v-if="!langSelected" @update:lang="on_lang_selected" />
  <DesktopOrInstall
    :class="'lang-' + current_lang"
    v-if="langSelected"
  />
  <!-- main content -->
  <DKLayout
    :class="'lang-' + current_lang"
    :main_class="lightup_seq(1)"
    v-if="langSelected"
  >
    <RouterView @update:can_quit="(v: boolean) => (can_quit = v)" />
    <template #left>
      <div class="wrapper" :class="lightup_seq(1)">
        <nav :class="nav_menu_bold(0)">{{ $t("d.nav-0") }}</nav>
        <nav :class="nav_menu_bold(1)">{{ $t("d.nav-1") }}</nav>
        <nav :class="nav_menu_bold(2)">{{ $t("d.nav-2") }}</nav>
        <nav :class="nav_menu_bold(3)">{{ $t("d.nav-3") }}</nav>
      </div>
    </template>
  </DKLayout>
  <!-- status bar -->
  <div
    class="status-bar"
    v-if="langSelected"
    :class="[lightup_seq(4), 'lang-' + current_lang]"
  >
    <progress
      id="progressbar"
      :aria-label="$t('d.sr-progress')"
      :value="progress"
      max="100"
      class="progress-bar"
    ></progress>
  </div>
</template>

<style>
main {
  transition: opacity 0.3s;
}
</style>

<style scoped>
.hidden {
  opacity: 0;
}

div,
header {
  transition: opacity 0.3s;
}

.status-bar {
  position: absolute;
  bottom: 2em;
  left: 0;
  width: 100%;
  min-height: 5vh;
}

.progress-bar {
  appearance: none;
  display: block;
  background: var(--dk-inactive);
  border: 0;
  width: 100%;
  height: 10px;
}

.progress-bar[value]::-webkit-progress-value {
  background: var(--dk-accent);
  transition: width 200ms;
}

.progress-bar::-moz-progress-bar {
  background: var(--dk-accent);
}

.info-box {
  margin-top: 0.5em;
  float: left;
  margin-left: 0.3rem;
}

.eta-box {
  float: right;
  margin-top: 0.5em;
  margin-right: 0.5em;
}

.quit-button {
  float: right;
  cursor: pointer;
  appearance: none;
  background: transparent;
  border: 0;
}

.quit-button[disabled] {
  cursor: not-allowed;
  filter: grayscale(1);
}

header {
  line-height: 1.5;
  width: 30%;
}

nav {
  width: 100%;
  font-size: 12px;
  text-align: center;
  margin-top: 1rem;
}

@media (min-width: 1024px) {
  header {
    display: flex;
    place-items: start;
    padding-top: 0.5rem;
    padding-right: calc(var(--section-gap) / 2);
  }

  header .wrapper {
    display: flex;
    place-items: flex-start;
    flex-wrap: wrap;
  }

  nav {
    text-align: left;
    margin-left: calc(60px + 0.5em);
    font-size: 1rem;
    color: var(--dk-gray);

    padding: 0.3rem 0;
    margin-top: 1rem;
  }

  nav.active {
    color: #eeeeee;
    font-weight: bold;
  }
}
</style>
