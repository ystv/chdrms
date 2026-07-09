import { listUsersOptions } from '#/client/@tanstack/react-query.gen';
import { Button, Group, Stack, Table, Title } from '@mantine/core';
import { useQuery } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router';
import { CheckIcon, XIcon } from 'lucide-react';

export const Route = createFileRoute('/(authenticated)/users/')({
  component: RouteComponent,
});

function RouteComponent() {
  const users = useQuery({ ...listUsersOptions() });

  return (
    <Stack>
      <Group>
        <Title>Users</Title>
      </Group>
      <Table striped>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Name</Table.Th>
            <Table.Th>Email</Table.Th>
            <Table.Th>Is Admin</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {users.data?.map((user) => (
            <Table.Tr key={user.id}>
              <Table.Td>{user.name}</Table.Td>
              <Table.Td>{user.email}</Table.Td>
              <Table.Td>
                {user.is_admin ? (
                  <CheckIcon color="green" />
                ) : (
                  <XIcon color="red" />
                )}
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
    </Stack>
  );
}
