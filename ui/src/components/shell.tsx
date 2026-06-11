import { AppShell, Burger, Group, NavLink, Stack, Text } from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import { Link } from '@tanstack/react-router';
import { XIcon } from 'lucide-react';
import type { ReactNode } from 'react';

export function Shell(props: { children: ReactNode }) {
  const [mobileOpened, { toggle: toggleMobile, close: closeMobile }] =
    useDisclosure();
  const [desktopOpened, { toggle: toggleDesktop }] = useDisclosure(true);

  return (
    <AppShell
      padding="md"
      header={{ height: 60 }}
      navbar={{
        width: 300,
        breakpoint: 'sm',
        collapsed: { mobile: !mobileOpened, desktop: !desktopOpened },
      }}
    >
      <AppShell.Header>
        <Group h="100%" px="md">
          <Burger
            opened={mobileOpened}
            onClick={toggleMobile}
            hiddenFrom="sm"
            size="sm"
          />
          <Burger
            opened={desktopOpened}
            onClick={toggleDesktop}
            visibleFrom="sm"
            size="sm"
          />
        </Group>
      </AppShell.Header>
      <AppShell.Navbar p="md">
        <NavLink component={Link} to="/" label="Home" />
        <NavLink
          component={Link}
          to="/test"
          label="Test"
          onClick={closeMobile}
        />
      </AppShell.Navbar>
      <AppShell.Main>{props.children}</AppShell.Main>
    </AppShell>
  );
}
