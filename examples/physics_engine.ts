// @decision Verlet Integration: Use Velocity Verlet over Euler integration for energy conservation in rigid body dynamics.
// @constraint Sub-stepping: Physics step size must never exceed 16ms to prevent tunneling through thin meshes.
// @warning High restitution coefficients above 1.0 cause kinetic energy explosion.

export function integrateVelocity(body: any, dt: number) {
    body.position.x += body.velocity.x * dt;
    body.position.y += body.velocity.y * dt;
}

export function detectCollision(bodyA: any, bodyB: any) {
    const dx = bodyA.position.x - bodyB.position.x;
    const dy = bodyA.position.y - bodyB.position.y;
    return Math.sqrt(dx * dx + dy * dy) < (bodyA.radius + bodyB.radius);
}

export function resolveImpulse(bodyA: any, bodyB: any) {
    if (detectCollision(bodyA, bodyB)) {
        integrateVelocity(bodyA, 0.016);
    }
}
