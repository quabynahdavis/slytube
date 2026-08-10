<script setup lang="ts">
import { ContextMenuTrigger, useForwardProps } from "reka-ui";
import type { ContextMenuTriggerProps } from "reka-ui";

interface Props extends ContextMenuTriggerProps {}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <ContextMenuTrigger v-bind="forwarded">
    <slot />
  </ContextMenuTrigger>
</template>
