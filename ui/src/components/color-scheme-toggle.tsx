'use client';

import type { MantineColorScheme, SegmentedControlProps } from '@mantine/core';
import {
  Center,
  SegmentedControl,
  useMantineColorScheme,
  VisuallyHidden,
} from '@mantine/core';
import { LaptopIcon, MoonIcon, SunIcon } from 'lucide-react';

export function ColorSchemeToggle(
  props: Omit<SegmentedControlProps, 'value' | 'onChange' | 'data'>,
) {
  const { setColorScheme, colorScheme } = useMantineColorScheme();

  return (
    <SegmentedControl
      {...props}
      value={colorScheme}
      onChange={(v: string) => setColorScheme(v as MantineColorScheme)}
      className="min-w-full"
      data={[
        {
          value: 'light',
          label: (
            <Center>
              <SunIcon aria-label="light mode" />
              <VisuallyHidden>Light Mode</VisuallyHidden>
            </Center>
          ),
        },
        {
          value: 'auto',
          label: (
            <Center>
              <LaptopIcon aria-label="system theme" />
              <VisuallyHidden>System Theme</VisuallyHidden>
            </Center>
          ),
        },
        {
          value: 'dark',
          label: (
            <Center>
              <MoonIcon aria-label="dark mode" />
              <VisuallyHidden>Dark Mode</VisuallyHidden>
            </Center>
          ),
        },
      ]}
    />
  );
}
