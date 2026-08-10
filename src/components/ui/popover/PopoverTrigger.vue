<script setup lang="ts">
import { PopoverTrigger, useForwardProps } from "reka-ui";
import type { PopoverTriggerProps } from "reka-ui";

interface Props extends PopoverTriggerProps {}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <PopoverTrigger v-bind="forwarded">
    <slot />
  </PopoverTrigger>
</template>
