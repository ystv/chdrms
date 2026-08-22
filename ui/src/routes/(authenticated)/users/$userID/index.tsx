import { getUserById } from '#/client';
import { getUserByIdOptions } from '#/client/@tanstack/react-query.gen';
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
import { createFileRoute, notFound } from '@tanstack/react-router';
import z from 'zod';

export const Route = createFileRoute('/(authenticated)/users/$userID/')({
  params: {
    parse: (params) => ({
      userID: z.uuid().parse(params.userID),
    }),
  },
  loader: async ({ params: { userID } }) => {
    const user = await getUserById({ path: { id: userID } });

    if (!user.data) {
      throw notFound();
    }

    return { user };
  },
  component: RouteComponent,
});

function RouteComponent() {
  const { userID } = Route.useParams();
  const { user: initialUser } = Route.useLoaderData();

  const user = useQuery({
    ...getUserByIdOptions({ path: { id: userID } }),
    initialData: initialUser.data,
    retry: false,
  });

  const clipboard = useClipboard({ timeout: 1000 });

  return (
    <Card>
      <Stack>
        <Group>
          <Title>{user.data.name}</Title>
          <Tooltip label={clipboard.copied ? 'Copied!' : 'Copy'}>
            <Button
              ml={'auto'}
              c={'dimmed'}
              variant="transparent"
              onClick={() => clipboard.copy(user.data.id)}
            >
              {user.data.id}
            </Button>
          </Tooltip>
        </Group>
        <Text>{user.data.email}</Text>
        {user.data.is_admin && 'This user is an admin'}
      </Stack>
    </Card>
  );
}
