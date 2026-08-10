<script setup lang="ts">
import { TooltipProvider, useForwardProps } from "reka-ui";
import type { TooltipProviderProps } from "reka-ui";

interface Props extends TooltipProviderProps {}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <TooltipProvider v-bind="forwarded">
    <slot />
  </TooltipProvider>
</template>
