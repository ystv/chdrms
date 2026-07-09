import {
  AppShell,
  Burger,
  Group,
  LoadingOverlay,
  NavLink,
  Text,
} from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import { Link, useNavigate } from '@tanstack/react-router';
import type { ReactNode } from 'react';
import { ColorSchemeToggle } from './color-scheme-toggle';
import { getCurrentUserOptions } from '#/client/@tanstack/react-query.gen';
import { useQuery } from '@tanstack/react-query';
import { HomeIcon, UserIcon, CirclePileIcon, UsersIcon } from 'lucide-react';
import LogoutButton from './logout-button';

export function Shell(props: { children: ReactNode }) {
  const [mobileOpened, { toggle: toggleMobile, close: closeMobile }] =
    useDisclosure();
  const [desktopOpened, { toggle: toggleDesktop }] = useDisclosure(true);

  const navigate = useNavigate();

  const me = useQuery({ ...getCurrentUserOptions({}), retry: false });

  if (!me.data && !me.error)
    return (
      <div style={{ width: '100%', height: '100vh' }}>
        <LoadingOverlay visible />
      </div>
    );

  if (me.error) {
    navigate({ to: '/login' });
  }

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
          <Text>{me.data?.name}</Text>
        </Group>
      </AppShell.Header>
      <AppShell.Navbar p="md">
        <NavLink
          component={Link}
          to="/"
          label="Home"
          leftSection={<HomeIcon />}
        />
        <NavLink
          component={Link}
          to="/users"
          label="Users"
          onClick={closeMobile}
          leftSection={<UsersIcon />}
        />
        <NavLink
          component={Link}
          to="/groups"
          label="Groups"
          onClick={closeMobile}
          leftSection={<CirclePileIcon />}
        />
        {me.data && (
          <NavLink
            component={Link}
            to="/users/@me"
            label="My Profile"
            onClick={closeMobile}
            leftSection={<UserIcon />}
          />
        )}
        <ColorSchemeToggle mt={'auto'} />
        <LogoutButton />
      </AppShell.Navbar>
      <AppShell.Main>{props.children}</AppShell.Main>
    </AppShell>
  );
}
