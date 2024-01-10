# Charge
Charge is an application that provides Seccomp Supervisor facilities for Kubernetes and bare-metal running applications.
It monitors syscalls made by applications and writes the logged syscalls in a JSON file for the GUI to evaluate.
The GUI provides explanations on the syscalls and the ability to deny or accept specific syscalls by the operator which will then be applied to a Seccomp profile for professional use.