<script setup lang="ts">
import { DropdownMenuTrigger, useForwardProps } from "reka-ui";
import type { DropdownMenuTriggerProps } from "reka-ui";

interface Props extends DropdownMenuTriggerProps {}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <DropdownMenuTrigger v-bind="forwarded">
    <slot />
  </DropdownMenuTrigger>
</template>
