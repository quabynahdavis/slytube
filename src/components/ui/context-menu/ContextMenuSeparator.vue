<script setup lang="ts">
import { ContextMenuSeparator, useForwardProps } from "reka-ui";
import type { ContextMenuSeparatorProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends ContextMenuSeparatorProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <ContextMenuSeparator
    v-bind="forwarded"
    :class="cn('-mx-1 my-1 h-px bg-border', props.class)"
  />
</template>
