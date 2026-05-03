import { useState } from 'react'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import { Plus } from 'lucide-react'

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
import { Form } from '@/shared/ui/form'
import { FormInput } from '@/shared/ui/form-input'
import { Checkbox } from '@/shared/ui/checkbox'
import { Separator } from '@/shared/ui/separator'
import { useCreateUser } from '@/features/user/api/useCreateUser'

const emailSchema = z
  .string()
  .trim()
  .optional()
  .refine(
    (val) => !val || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(val),
    { message: 'Invalid email address' }
  )

const createFormSchema = z.object({
  username: z.string().min(3),
  email: emailSchema,
  password: z.string().min(8),
  ignore_password_policies: z.boolean(),
})

const inviteFormSchema = z.object({
  email: z
    .string()
    .trim()
    .optional()
    .refine(
      (val) => !val || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(val),
      { message: 'Invalid email address' }
    ),
  expiry_days: z.number().min(1),
})

type CreateFormValues = z.infer<typeof createFormSchema>
type InviteFormValues = z.infer<typeof inviteFormSchema>

export function CreateUserDialog() {
  const [open, setOpen] = useState(false)
  const [activeTab, setActiveTab] = useState('create')
  
  const mutation = useCreateUser()
  
  const createForm = useForm<CreateFormValues>({
    resolver: zodResolver(createFormSchema),
    defaultValues: { username: '', email: '', password: '', ignore_password_policies: false },
  })

  const inviteForm = useForm<InviteFormValues>({
    resolver: zodResolver(inviteFormSchema),
    defaultValues: { email: '', expiry_days: 7 },
  })

  const handleOpenChange = (newOpen: boolean) => {
    setOpen(newOpen)
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
      { onSuccess: () => handleOpenChange(false) }
    )
  }

  const onInviteSubmit = (values: InviteFormValues) => {
    // TODO: Implement actual invite endpoint here
    console.log('Sending invite...', values)
    handleOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogTrigger asChild>
        <Button size="sm" className="flex items-center gap-2">
          <Plus size={18} />
          <span>Create User</span>
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>{activeTab === 'create' ? 'Create new user' : 'Invite new user'}</DialogTitle>
        </DialogHeader>
        
        <Separator className="my-1" />

        <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full mt-2">
          <TabsList variant="line" className="mb-4">
            <TabsTrigger variant="line" value="create">Create user</TabsTrigger>
            <TabsTrigger variant="line" value="invite">Invite User</TabsTrigger>
          </TabsList>
          
          <TabsContent value="create">
            <Form {...createForm}>
              <div className="grid gap-4 py-4">
                <FormInput control={createForm.control} name="username" label="Username" />
                <FormInput control={createForm.control} name="email" label="Email (Optional)" type="email" />
                <FormInput control={createForm.control} name="password" label="Password" type="password" />
                
                <div className="flex items-start space-x-3 mt-2">
                  <Checkbox 
                    id="ignore_policies" 
                    checked={createForm.watch('ignore_password_policies')}
                    onCheckedChange={(checked) => createForm.setValue('ignore_password_policies', checked as boolean)}
                  />
                  <div className="grid gap-1.5 leading-none -mt-0.5">
                    <label
                      htmlFor="ignore_policies"
                      className="text-[14px] font-medium leading-none cursor-pointer"
                    >
                      Ignore Password Policies
                    </label>
                    <p className="text-[13px] text-muted-foreground mt-1">
                      If checked, password policies will not be enforced on this password.
                    </p>
                  </div>
                </div>
              </div>
              <DialogFooter>
                <Button variant="outline" type="button" onClick={() => handleOpenChange(false)}>
                  Cancel
                </Button>
                <Button onClick={createForm.handleSubmit(onCreateSubmit)} disabled={mutation.isPending}>
                  {mutation.isPending ? 'Creating...' : 'Create User'}
                </Button>
              </DialogFooter>
            </Form>
          </TabsContent>

          <TabsContent value="invite">
            <Form {...inviteForm}>
              <div className="grid gap-4 py-4">
                <FormInput control={inviteForm.control} name="email" label="Email address" type="email" />
                <FormInput 
                  control={inviteForm.control} 
                  name="expiry_days" 
                  label="Set invitation expiry (days)" 
                  type="number"
                  min={1} 
                />
                <p className="text-[13px] text-muted-foreground -mt-2">
                  Invite links will expire after the specified number of days.
                </p>
              </div>
              <DialogFooter>
                <Button variant="outline" type="button" onClick={() => handleOpenChange(false)}>
                  Cancel
                </Button>
                <Button onClick={inviteForm.handleSubmit(onInviteSubmit)}>
                  Send Invite
                </Button>
              </DialogFooter>
            </Form>
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  )
}
