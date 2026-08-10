<script setup lang="ts">
import { TooltipTrigger, useForwardProps } from "reka-ui";
import type { TooltipTriggerProps } from "reka-ui";

interface Props extends TooltipTriggerProps {}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <TooltipTrigger v-bind="forwarded">
    <slot />
  </TooltipTrigger>
</template>
