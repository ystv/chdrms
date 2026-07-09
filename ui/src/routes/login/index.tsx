import {
  getCurrentUserOptions,
  listAuthProvidersOptions,
} from '#/client/@tanstack/react-query.gen';
import { Button, Card, Center, Stack, Title } from '@mantine/core';
import { useQuery } from '@tanstack/react-query';
import { createFileRoute, useNavigate } from '@tanstack/react-router';

export const Route = createFileRoute('/login/')({
  component: RouteComponent,
});

function RouteComponent() {
  const providers = useQuery({ ...listAuthProvidersOptions() });

  const navigate = useNavigate();

  const me = useQuery({ ...getCurrentUserOptions({}), retry: false });

  if (!me.isFetching && !me.isRefetching && me.status === 'success') {
    console.log('User logged in, redirecting to home page');
    navigate({ to: '/' });
  }

  return (
    <Center h={'100vh'}>
      <Card>
        <Stack>
          <Center>
            <Title>Login</Title>
          </Center>
          {providers.data?.map((provider) => (
            <Button
              component={'a'}
              href={`/auth/${provider.id}/begin`}
              key={provider.id}
            >
              {provider.name}
            </Button>
          ))}
        </Stack>
      </Card>
    </Center>
  );
}
