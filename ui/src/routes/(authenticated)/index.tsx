import { Button, Title } from '@mantine/core';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/(authenticated)/')({ component: Home });

function Home() {
  return <Title>CHDRMS</Title>;
}
