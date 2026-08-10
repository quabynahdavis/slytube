<script setup lang="ts">
import { DialogTrigger, useForwardProps } from "reka-ui";
import type { DialogTriggerProps } from "reka-ui";

interface Props extends DialogTriggerProps {}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <DialogTrigger v-bind="forwarded">
    <slot />
  </DialogTrigger>
</template>
