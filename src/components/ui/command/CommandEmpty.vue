<script setup lang="ts">
import { ComboboxEmpty, useForwardProps } from "reka-ui";
import type { ComboboxEmptyProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends ComboboxEmptyProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <ComboboxEmpty
    v-bind="forwarded"
    :class="cn('py-6 text-center text-sm', props.class)"
  >
    <slot />
  </ComboboxEmpty>
</template>
