<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import DKBottomActions from '@/components/DKBottomActions.vue';
import DKBottomRightButtons from '@/components/DKBottomRightButtons.vue';
import { defineComponent } from 'vue';
import DKBody from '../components/DKBody.vue';
</script>

<script lang="ts">
export default defineComponent({
  methods: {
    async finish() {
      await invoke('exit')
    },
  },
});
</script>

<template>
  <DKBody>
    <div>
      <h1>{{ $t("finish.title") }}</h1>
      <p>{{ $t("finish.p1") }}</p>
      <p>{{ $t("finish.p2") }}</p>
      <p>{{ $t("finish.p3") }}</p>
    </div>
  </DKBody>
  <DKBottomActions>
    <DKBottomRightButtons>
      <button class="button" @click="finish">{{ $t("finish.reboot") }}</button>
    </DKBottomRightButtons>
  </DKBottomActions>
</template>
