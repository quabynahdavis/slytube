<script setup lang="ts">
import { DropdownMenuSeparator, useForwardProps } from "reka-ui";
import type { DropdownMenuSeparatorProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends DropdownMenuSeparatorProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <DropdownMenuSeparator
    v-bind="forwarded"
    :class="cn('-mx-1 my-1 h-px bg-border', props.class)"
  />
</template>
