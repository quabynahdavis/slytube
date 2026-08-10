<script setup lang="ts">
import { AlertDialogTrigger, useForwardProps } from "reka-ui";
import type { AlertDialogTriggerProps } from "reka-ui";

interface Props extends AlertDialogTriggerProps {}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <AlertDialogTrigger v-bind="forwarded">
    <slot />
  </AlertDialogTrigger>
</template>
