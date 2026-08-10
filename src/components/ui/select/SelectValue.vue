<script setup lang="ts">
import { SelectValue, useForwardProps } from "reka-ui";
import type { SelectValueProps } from "reka-ui";

interface Props extends SelectValueProps {}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <SelectValue v-bind="forwarded">
    <slot />
  </SelectValue>
</template>
