<script setup lang="ts">
import { AvatarImage, useForwardProps } from "reka-ui";
import type { AvatarImageProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends AvatarImageProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <AvatarImage
    v-bind="forwarded"
    :class="cn('aspect-square h-full w-full', props.class)"
  />
</template>
