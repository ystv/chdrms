import { Button } from '@mantine/core';
import type { ButtonProps } from '@mantine/core';
import { useNavigate } from '@tanstack/react-router';

export default function LogoutButton(props: ButtonProps) {
  const navigate = useNavigate();

  return (
    <Button
      color="red"
      {...props}
      onClick={async () => {
        await fetch('/auth/logout', { method: 'POST' });

        navigate({ to: '/login' });
      }}
    >
      Logout
    </Button>
  );
}
