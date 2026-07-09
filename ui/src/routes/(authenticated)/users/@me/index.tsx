import { getCurrentUserOptions } from '#/client/@tanstack/react-query.gen';
import {
  Button,
  Card,
  Group,
  Stack,
  Text,
  Title,
  Tooltip,
} from '@mantine/core';
import { useClipboard } from '@mantine/hooks';
import { useQuery } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/(authenticated)/users/@me/')({
  component: RouteComponent,
});

function RouteComponent() {
  const me = useQuery({ ...getCurrentUserOptions({}) });

  const clipboard = useClipboard({ timeout: 1000 });

  return (
    <Card>
      <Stack>
        <Group>
          <Title>{me.data?.name}</Title>
          <Tooltip label={clipboard.copied ? 'Copied!' : 'Copy'}>
            <Button
              ml={'auto'}
              c={'dimmed'}
              variant="transparent"
              onClick={() => clipboard.copy(me.data?.id)}
            >
              {me.data?.id}
            </Button>
          </Tooltip>
        </Group>
        <Text>{me.data?.email}</Text>
        {me.data?.is_admin && 'This user is an admin'}
      </Stack>
    </Card>
  );
}
