<script setup lang="ts">
import { DropdownMenuRoot, useForwardPropsEmits } from "reka-ui";
import type { DropdownMenuRootEmits, DropdownMenuRootProps } from "reka-ui";

interface Props extends DropdownMenuRootProps {}

const props = defineProps<Props>();
const emits = defineEmits<DropdownMenuRootEmits>();

const forwarded = useForwardPropsEmits(props, emits);
</script>

<template>
  <DropdownMenuRoot v-bind="forwarded">
    <slot />
  </DropdownMenuRoot>
</template>
