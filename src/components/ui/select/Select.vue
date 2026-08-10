<script setup lang="ts">
import { SelectRoot, useForwardPropsEmits } from "reka-ui";
import type { SelectRootEmits, SelectRootProps } from "reka-ui";

interface Props extends SelectRootProps {}

const props = defineProps<Props>();
const emits = defineEmits<SelectRootEmits>();

const forwarded = useForwardPropsEmits(props, emits);
</script>

<template>
  <SelectRoot v-bind="forwarded">
    <slot />
  </SelectRoot>
</template>
