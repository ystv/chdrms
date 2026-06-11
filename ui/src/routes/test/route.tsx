import { Title } from '@mantine/core';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/test')({ component: Test });

function Test() {
  return <Title>Test Page</Title>;
}
