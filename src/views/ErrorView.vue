<script lang="ts">
import DKBottomActions from '@/components/DKBottomActions.vue';
import DKBottomRightButtons from '@/components/DKBottomRightButtons.vue';
import { defineComponent } from 'vue';

export default defineComponent({
  props: {
    message: { type: String, required: true },
    openGparted: Number,
    currentRoute: { type: String, required: true },
  },
  data() {
    return {
      loading: false,
    };
  },
  components: { DKBottomActions, DKBottomRightButtons },
  methods: {},
});
</script>

<template>
  <div v-if="!loading">
    <div>
      <h1>{{ $t("error.title") }}</h1>
      <p>{{ $t("error.p1") }}</p>
      <p class="error-msg">{{ decodeURIComponent(message) }}</p>
    </div>
  </div>
  <div v-else>
    <div class="loading">
      <h1>{{ $t("error.title") }}</h1>
      <DKSpinner :title="$t('error.retrying')" />
    </div>
  </div>
</template>
