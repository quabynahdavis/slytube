<script setup lang="ts">
import { ToastProvider, useForwardProps } from "reka-ui";
import type { ToastProviderProps } from "reka-ui";

interface Props extends ToastProviderProps {}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <ToastProvider v-bind="forwarded">
    <slot />
  </ToastProvider>
</template>
