<script setup lang="ts">
import { AlertDialogRoot, useForwardPropsEmits } from "reka-ui";
import type { AlertDialogEmits, AlertDialogProps } from "reka-ui";

interface Props extends AlertDialogProps {}

const props = defineProps<Props>();
const emits = defineEmits<AlertDialogEmits>();

const forwarded = useForwardPropsEmits(props, emits);
</script>

<template>
  <AlertDialogRoot v-bind="forwarded">
    <slot />
  </AlertDialogRoot>
</template>
