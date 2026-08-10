<script setup lang="ts">
import { ComboboxContent, useForwardProps } from "reka-ui";
import type { ComboboxContentProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends ComboboxContentProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <ComboboxContent
    v-bind="forwarded"
    :class="cn('max-h-[300px] overflow-y-auto overflow-x-hidden', props.class)"
  >
    <slot />
  </ComboboxContent>
</template>
