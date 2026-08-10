<script setup lang="ts">
import { TabsList, useForwardProps } from "reka-ui";
import type { TabsListProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends TabsListProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <TabsList
    v-bind="forwarded"
    :class="
      cn(
        'inline-flex h-9 items-center justify-center rounded-lg bg-muted p-1 text-muted-foreground',
        props.class
      )
    "
  >
    <slot />
  </TabsList>
</template>
