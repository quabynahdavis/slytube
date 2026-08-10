<script setup lang="ts">
import { DialogRoot, useForwardPropsEmits } from "reka-ui";
import type { DialogRootEmits, DialogRootProps } from "reka-ui";

interface Props extends DialogRootProps {}

const props = defineProps<Props>();
const emits = defineEmits<DialogRootEmits>();

const forwarded = useForwardPropsEmits(props, emits);
</script>

<template>
  <DialogRoot v-bind="forwarded">
    <slot />
  </DialogRoot>
</template>
