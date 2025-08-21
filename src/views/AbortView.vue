<script setup lang="ts">
import DKSpinner from '@/components/DKSpinner.vue';
</script>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  props: {
    done: Number,
  },
  data() {
    return {
      exiting: false,
    };
  },
  methods: {
  },
  async mounted() {
  },
});
</script>

<template>
  <div v-if="!exiting && done !== 1">
    <h1>{{ $t("abort.title") }}</h1>
    <i18n-t keypath="abort.p1" tag="p">
      {{ $t("abort.p1-1") }}
    </i18n-t>
    <p>{{ $t("abort.p2") }}</p>
  </div>
  <div class="loading" v-else>
    <h1>{{ $t("exiting.t1") }}</h1>
    <DKSpinner :title="$t('exiting.l1')" />
  </div>
</template>
