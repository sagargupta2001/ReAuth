import React, { useState } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Plus } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { Button } from '@/components/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/dialog'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { useCreateInvitation } from '@/features/invitation/api/useInvitations'
import { useCreateUser } from '@/features/user/api/useCreateUser'
import { ApiError } from '@/shared/api/client'
import { ButtonGroup } from '@/shared/ui/button-group'
import { Checkbox } from '@/shared/ui/checkbox'
import { Form } from '@/shared/ui/form'
import { FormInput } from '@/shared/ui/form-input'
import { Separator } from '@/shared/ui/separator'

const emailSchema = z
  .string()
  .trim()
  .optional()
  .refine((val) => !val || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(val), {
    message: 'Invalid email address',
  })

const createFormSchema = z
  .object({
    username: z.string().min(3),
    email: emailSchema,
    password: z.string(),
    ignore_password_policies: z.boolean(),
  })
  .superRefine((data, ctx) => {
    if (!data.ignore_password_policies && data.password.length < 8) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: 'Password must be at least 8 characters',
        path: ['password'],
      })
    }
  })

const inviteFormSchema = z.object({
  email: z
    .string()
    .trim()
    .min(1, { message: 'Email is required' })
    .email({ message: 'Invalid email address' }),
  expiry_days: z.number().min(1),
})

type CreateFormValues = z.infer<typeof createFormSchema>
type InviteFormValues = z.infer<typeof inviteFormSchema>

interface CreateUserDialogProps {
  open?: boolean
  onOpenChange?: (open: boolean) => void
}

export function CreateUserDialog({ open: controlledOpen, onOpenChange: controlledOnOpenChange }: CreateUserDialogProps = {}) {
  const isControlled = controlledOpen !== undefined
  const [internalOpen, setInternalOpen] = useState(false)
  const open = isControlled ? controlledOpen : internalOpen
  const [activeTab, setActiveTab] = useState('create')

  const mutation = useCreateUser()
  const inviteMutation = useCreateInvitation()

  const createForm = useForm<CreateFormValues>({
    resolver: zodResolver(createFormSchema),
    defaultValues: { username: '', email: '', password: '', ignore_password_policies: false },
  })

  const inviteForm = useForm<InviteFormValues>({
    resolver: zodResolver(inviteFormSchema),
    defaultValues: { email: '', expiry_days: 7 },
  })

  const handleOpenChange = (newOpen: boolean) => {
    if (!isControlled) setInternalOpen(newOpen)
    controlledOnOpenChange?.(newOpen)
    if (!newOpen) {
      createForm.reset()
      inviteForm.reset()
      setActiveTab('create')
    }
  }

  const onCreateSubmit = (values: CreateFormValues) => {
    const email = values.email?.trim() || undefined
    mutation.mutate(
      { ...values, email },
      {
        onSuccess: () => handleOpenChange(false),
        onError: (error) => {
          if (
            error instanceof ApiError &&
            error.body &&
            typeof error.body === 'object' &&
            'fields' in error.body &&
            error.body.fields &&
            typeof error.body.fields === 'object'
          ) {
            const fields = error.body.fields as Record<string, string>
            Object.entries(fields).forEach(([field, message]) => {
              createForm.setError(field as keyof CreateFormValues, {
                type: 'server',
                message,
              })
            })
          }
        },
      },
    )
  }

  const onInviteSubmit = (values: InviteFormValues) => {
    inviteMutation.mutate(
      {
        email: values.email.trim(),
        expiry_days: values.expiry_days,
      },
      {
        onSuccess: () => handleOpenChange(false),
        onError: (error) => {
          if (error instanceof ApiError) {
            inviteForm.setError('email', {
              type: 'server',
              message: error.message,
            })
          }
        },
      },
    )
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      {!isControlled && (
        <DialogTrigger asChild>
          <Button size="sm" className="flex items-center gap-2">
            <Plus size={18} />
            <span>Create User</span>
          </Button>
        </DialogTrigger>
      )}
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader className="pt-6 pl-6">
          <DialogTitle>
            {activeTab === 'create' ? 'Create new user' : 'Invite new user'}
          </DialogTitle>
        </DialogHeader>

        <Separator className="my-1" />

        <Tabs value={activeTab} onValueChange={setActiveTab} className="mt-2 w-full">
          <TabsList variant="line" className="mb-4 px-6">
            <TabsTrigger variant="line" value="create">
              Create user
            </TabsTrigger>
            <TabsTrigger variant="line" value="invite">
              Invite User
            </TabsTrigger>
          </TabsList>

          <TabsContent value="create">
            <Form {...createForm}>
              <div className="grid gap-4 px-6 pb-6">
                <FormInput
                  control={createForm.control}
                  name="username"
                  label="Username"
                  placeholder="Enter username"
                />
                <FormInput
                  control={createForm.control}
                  name="email"
                  label="Email (Optional)"
                  type="email"
                  placeholder="name@example.com"
                />
                <FormInput
                  control={createForm.control}
                  name="password"
                  label="Password"
                  type="password"
                  placeholder="Enter password"
                />

                <div className="mt-2 flex items-start space-x-3">
                  <Checkbox
                    id="ignore_policies"
                    checked={createForm.watch('ignore_password_policies')}
                    onCheckedChange={(checked) =>
                      createForm.setValue('ignore_password_policies', checked as boolean)
                    }
                  />
                  <div className="-mt-0.5 grid gap-1.5 leading-none">
                    <label
                      htmlFor="ignore_policies"
                      className="cursor-pointer text-[14px] leading-none font-medium"
                    >
                      Ignore Password Policies
                    </label>
                    <p className="text-muted-foreground mt-1 text-[13px]">
                      If checked, password policies will not be enforced on this password.
                    </p>
                  </div>
                </div>
              </div>
              <DialogFooter className="gap-1 py-3 pr-3">
                <Button variant="outline" type="button" onClick={() => handleOpenChange(false)}>
                  Cancel
                </Button>
                <Button
                  size="sm"
                  onClick={createForm.handleSubmit(onCreateSubmit)}
                  disabled={mutation.isPending}
                >
                  {mutation.isPending ? 'Creating...' : 'Create User'}
                </Button>
              </DialogFooter>
            </Form>
          </TabsContent>

          <TabsContent value="invite">
            <Form {...inviteForm}>
              <div className="grid gap-4 px-6 pb-6">
                <FormInput
                  control={inviteForm.control}
                  name="email"
                  label="Email address"
                  type="email"
                  placeholder="name@example.com"
                />

                <FormInput
                  control={inviteForm.control}
                  name="expiry_days"
                  label="Set invitation expiry"
                  description="Invite links will expire after the specified number of days."
                  type="number"
                  min={1}
                  placeholder="7"
                  className="no-number-arrows w-24"
                  onChange={(e) =>
                    inviteForm.setValue('expiry_days', parseInt(e.target.value) || 0)
                  }
                  render={(input: React.ReactNode) => (
                    <ButtonGroup>
                      {input}
                      <Button variant="outline" className="bg-muted pointer-events-none px-4">
                        Days
                      </Button>
                    </ButtonGroup>
                  )}
                />
              </div>
              <DialogFooter className="gap-1 py-3 pr-3">
                <Button variant="outline" onClick={() => handleOpenChange(false)}>
                  Cancel
                </Button>
                <Button
                  size="sm"
                  onClick={inviteForm.handleSubmit(onInviteSubmit)}
                  disabled={inviteMutation.isPending}
                >
                  {inviteMutation.isPending ? 'Sending...' : 'Send Invite'}
                </Button>
              </DialogFooter>
            </Form>
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  )
}
